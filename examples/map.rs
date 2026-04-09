use std::{
    collections::{BTreeMap, BTreeSet},
    future::ready,
    ops::Deref,
    sync::Arc,
};

use object_rainbow::{
    Address, ByteNode, FailFuture, Fetch, FullHash, Hash, Object, Point, PointVisitor, Resolve,
    Singular, Traversible, error_parse,
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
    async fn send(self, object: impl Traversible) {
        let send = self.send.clone();
        let event = Event::from_object(object, self);
        let _ = send.send(event).await;
    }

    async fn resolve(self, point: Point<impl Traversible>) {
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
struct MapResolver(Arc<BTreeMap<Hash, Vec<u8>>>);

impl MapResolver {
    fn resolve_bytes(&self, address: Address) -> object_rainbow::Result<Vec<u8>> {
        match self.0.get(&address.hash) {
            Some(data) => Ok(data.clone()),
            None => Err(error_parse!("hash not found")),
        }
    }
}

impl Resolve for MapResolver {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(
            self.resolve_bytes(address)
                .map(|data| (data, Arc::new(self.clone()) as _)),
        ))
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(self.resolve_bytes(address)))
    }

    fn name(&self) -> &str {
        "map resolver"
    }
}

async fn iterate<T: Object>(point: Point<T>) -> anyhow::Result<Point<T>> {
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
    let resolver = Arc::new(MapResolver(Arc::new(fetched)));
    Ok(point.with_resolve(resolver, ()))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting");
    smol::block_on(async move {
        let mut point =
            iterate(((*b"alisa", *b"feistel").point(), [1, 2, 3, 4].point()).point()).await?;
        assert_eq!(point.fetch().await?.0.fetch().await?.0, *b"alisa");
        assert_eq!(point.fetch().await?.0.fetch().await?.1, *b"feistel");
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 4]);
        println!("{}", hex::encode(point.full_hash()));
        point.fetch_mut().await?.1.fetch_mut().await?[3] = 5;
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 5]);
        println!("{}", hex::encode(point.full_hash()));
        point.fetch_mut().await?.1.fetch_mut().await?[3] = 4;
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 4]);
        println!("{}", hex::encode(point.full_hash()));
        Ok(())
    })
}
