use std::{
    collections::{BTreeMap, BTreeSet},
    future::ready,
    ops::Deref,
    sync::Arc,
};

use object_rainbow::{
    Address, ByteNode, FailFuture, Fetch, Hash, Object, Point, RefVisitor, Refless, Resolve,
    Singular,
};
use smol::{Executor, channel::Sender};

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
    async fn send(self, object: impl Object) {
        let send = self.send.clone();
        let event = Event::from_object(object, self);
        let _ = send.send(event).await;
    }

    async fn resolve(self, point: Point<impl Object>) {
        match point.fetch().await {
            Ok(object) => self.send(object).await,
            Err(e) => tracing::error!("{e:?}"),
        }
    }
}

impl RefVisitor for EventVisitor<'_, '_> {
    fn visit<T: Object>(&mut self, point: &object_rainbow::Point<T>) {
        if !self.fetching.contains(point.hash()) {
            self.fetching.insert(*point.hash());
            let point = point.clone();
            let context = self.context.clone();
            self.executor.spawn(context.resolve(point)).detach();
        }
    }
}

impl<'ex> Event<'ex> {
    fn from_object<T: Object>(object: T, context: EventContext<'ex>) -> Self {
        let hash = object.full_hash();
        let data = object.output();
        Event(
            hash,
            data,
            Box::new(move |fetching| object.accept_refs(&mut EventVisitor { fetching, context })),
        )
    }
}

#[derive(Debug, Clone)]
struct MapResolver(Arc<BTreeMap<Hash, Vec<u8>>>);

impl Resolve for MapResolver {
    fn resolve(&self, address: object_rainbow::Address) -> FailFuture<ByteNode> {
        Box::pin(ready(match self.0.get(&address.hash) {
            Some(data) => Ok((data.clone(), Arc::new(self.clone()) as _)),
            None => Err(object_rainbow::error!("hash not found")),
        }))
    }
}

async fn iterate<T: Object>(object: T) -> Point<T> {
    let (send, recv) = smol::channel::unbounded::<Event>();
    let executor = Arc::new(Executor::new());
    let hash = object.full_hash();
    EventContext {
        executor: executor.clone(),
        send,
    }
    .send(object)
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
    let address = Address { index: 0, hash };
    let resolver = Arc::new(MapResolver(Arc::new(fetched)));
    Point::from_address(address, resolver)
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting");
    smol::block_on(async move {
        let point = iterate((
            Point::from_object(Refless((*b"alisa", *b"feistel"))),
            Point::from_object(Refless([1, 2, 3, 4])),
        ))
        .await;
        assert_eq!(point.fetch().await?.0.fetch().await?.0.0, *b"alisa");
        assert_eq!(point.fetch().await?.0.fetch().await?.0.1, *b"feistel");
        assert_eq!(point.fetch().await?.1.fetch().await?.0, [1, 2, 3, 4]);
        Ok(())
    })
}
