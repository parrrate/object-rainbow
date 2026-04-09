use std::{
    collections::{BTreeMap, BTreeSet},
    future::ready,
    ops::Deref,
    sync::Arc,
};

use chacha20poly1305::{ChaCha20Poly1305, aead::Aead};
use object_rainbow::{
    ByteNode, FailFuture, Fetch, Hash, Object, Point, PointVisitor, Resolve, Singular, Traversible,
    error_fetch,
};
use object_rainbow_encrypted::{Key, WithKey, encrypt_point};
use sha2::digest::generic_array::GenericArray;
use smol::{Executor, channel::Sender};

#[derive(Debug, Clone, Copy)]
struct Test([u8; 32]);

impl Key for Test {
    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let cipher = {
            use chacha20poly1305::KeyInit;
            ChaCha20Poly1305::new(&self.0.into())
        };
        let nonce = &{
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize()
        };
        let nonce = &nonce.as_slice()[..12];
        let encrypted = cipher
            .encrypt(GenericArray::from_slice(nonce), data)
            .expect("we do not handle decryption errors");
        [nonce, encrypted.as_slice()].concat()
    }

    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>> {
        let cipher = {
            use chacha20poly1305::KeyInit;
            ChaCha20Poly1305::new(&self.0.into())
        };
        cipher
            .decrypt(GenericArray::from_slice(&data[..12]), &data[12..])
            .map_err(|_| error_fetch!("decryption failed"))
    }
}

type Callback<'a> = dyn 'a + Send + FnOnce(&mut BTreeSet<Hash>);

struct Event<'a>(Hash, Vec<u8>, Box<Callback<'a>>);

#[derive(Debug, Clone)]
struct EventContext<'ex> {
    executor: Arc<Executor<'ex>>,
    send: Sender<Event<'ex>>,
}

struct EventVisitor<'ex, 't> {
    fetching: &'t mut BTreeSet<Hash>,
    context: EventContext<'ex>,
}

impl<'ex> Deref for EventVisitor<'ex, '_> {
    type Target = EventContext<'ex>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl EventContext<'_> {
    async fn send(self, object: impl Traversible) {
        let send = self.send.clone();
        let event = Event::from_object(object, self);
        let _ = send.send(event).await;
    }

    async fn resolve(self, point: impl Fetch<T: Traversible>) {
        match point.fetch().await {
            Ok(object) => self.send(object).await,
            Err(e) => tracing::error!("{e:?}"),
        }
    }
}

impl PointVisitor for EventVisitor<'_, '_> {
    fn visit<T: Traversible>(&mut self, point: &object_rainbow::Point<T>) {
        if !self.fetching.contains(&point.hash()) {
            self.fetching.insert(point.hash());
            let point = point.clone();
            let context = self.context.clone();
            self.executor.spawn(context.resolve(point)).detach();
        }
    }
}

impl<'ex> Event<'ex> {
    fn from_object<T: Traversible>(object: T, context: EventContext<'ex>) -> Self {
        let hash = object.full_hash();
        let data = object.output();
        Event(
            hash,
            data,
            Box::new(move |fetching| object.accept_points(&mut EventVisitor { fetching, context })),
        )
    }
}

#[derive(Debug, Clone)]
struct MapResolve(Arc<BTreeMap<Hash, Vec<u8>>>);

impl Resolve for MapResolve {
    fn resolve(&'_ self, address: object_rainbow::Address) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(match self.0.get(&address.hash) {
            Some(data) => Ok((data.clone(), Arc::new(self.clone()) as _)),
            None => Err(object_rainbow::Error::HashNotFound),
        }))
    }

    fn resolve_data(&'_ self, address: object_rainbow::Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(match self.0.get(&address.hash) {
            Some(data) => Ok(data.clone()),
            None => Err(object_rainbow::Error::HashNotFound),
        }))
    }
}

async fn iterate<T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    point: Point<T>,
    extra: Extra,
) -> anyhow::Result<Point<T>> {
    let (send, recv) = smol::channel::unbounded::<Event>();
    let executor = Arc::new(Executor::new());
    EventContext {
        executor: executor.clone(),
        send,
    }
    .send(point.fetch().await?)
    .await;
    let fetched = executor
        .run(async {
            let mut fetching = BTreeSet::new();
            let mut fetched = BTreeMap::new();
            while let Ok(Event(hash, data, process)) = recv.recv().await {
                fetched.insert(hash, data);
                process(&mut fetching);
            }
            fetched
        })
        .await;
    for (k, v) in &fetched {
        println!(
            "{} {}",
            hex::encode(k),
            if v.iter().all(|x| *x >= b' ') {
                String::from_utf8(v.clone()).unwrap_or_else(|e| hex::encode(e.into_bytes()))
            } else {
                hex::encode(v)
            },
        );
    }
    let resolve = Arc::new(MapResolve(Arc::new(fetched)));
    Ok(point.with_resolve(resolve, extra))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting");
    smol::block_on(async move {
        let point = (
            (*b"alisa", *b"feistel").point().point(),
            [1, 2, 3, 4].point(),
        )
            .point();
        let key = Test(std::array::from_fn(|i| i as _));
        let point = encrypt_point(key, point).await?;
        let point = iterate(point, WithKey { key, extra: () }).await?;
        let point = point.fetch().await?.into_inner().point();
        let point = encrypt_point(key, point).await?;
        let point = point.fetch().await?.into_inner().point();
        assert_eq!(
            point.fetch().await?.0.fetch().await?.fetch().await?.0,
            *b"alisa",
        );
        assert_eq!(
            point.fetch().await?.0.fetch().await?.fetch().await?.1,
            *b"feistel",
        );
        println!("all right");
        Ok(())
    })
}
