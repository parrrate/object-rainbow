use std::{
    collections::{BTreeMap, btree_map},
    pin::pin,
    sync::Arc,
};

use async_executor::{Executor, Task};
use futures_channel::oneshot;
use futures_util::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use genawaiter_try_stream::try_stream;
use object_rainbow::{Address, FetchBytes, Hash, Resolve, Singular};

/// Commands coming from a consumer.
pub enum Consume {
    /// Request [`Provide::Deliver`]y. Requires refcount of at least 1.
    Order(Hash),
    /// Increase server-side refcount by 1. Requires refcount of at least 1.
    Inc(Hash),
    /// Decrease server-side refcount by 1. Requires refcount of at least 1.
    Dec(Hash),
    /// Increase child's refcount by 1. Requires parent refcount of at least 1.
    IncChild { parent: Hash, child: Address },
}

/// Responses coming from a provider.
pub enum Provide {
    /// Fulfil an [`Consume::Order`].
    Deliver(Hash, Vec<u8>),
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish { hash: Hash, reason: Vec<u8> },
}

enum ProviderEvent {
    Consumed(Consume),
    Published((Arc<dyn Singular>, Vec<u8>)),
}

type Fetching = Task<Result<(Vec<u8>, Arc<dyn Resolve>), object_rainbow::Error>>;

struct Retained {
    count: u128,
    point: Arc<dyn Singular>,
    fetching: Option<Fetching>,
    resolve: Option<Arc<dyn Resolve>>,
    ordered: bool,
    waiting: Vec<oneshot::Sender<Arc<dyn Resolve>>>,
}

impl Retained {
    async fn finish_fetch(&mut self) -> object_rainbow::Result<Option<Vec<u8>>> {
        let (data, resolve) = self
            .fetching
            .take()
            .ok_or_else(|| object_rainbow::error_consistency!("not currently fetching"))?
            .await?;
        for waiting in std::mem::take(&mut self.waiting) {
            waiting.send(resolve.clone()).ok();
        }
        self.resolve = Some(resolve);
        Ok(if std::mem::take(&mut self.ordered) {
            Some(data)
        } else {
            None
        })
    }

    fn start_fetch(&mut self, executor: &Executor) -> object_rainbow::Result<()> {
        if self.fetching.is_none() {
            let point = self.point.clone();
            self.fetching = Some(executor.spawn(async move { point.fetch_bytes().await }));
            Ok(())
        } else {
            Err(object_rainbow::error_consistency!("already fetching"))
        }
    }

    #[expect(dead_code)]
    fn ensure_fetch(&mut self, executor: &Executor) -> object_rainbow::Result<()> {
        if self.fetching.is_none() {
            self.start_fetch(executor)?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct Retain(BTreeMap<Hash, Retained>);

impl Retain {
    #[expect(dead_code)]
    async fn finish_fetch(&mut self, hash: Hash) -> object_rainbow::Result<Option<Vec<u8>>> {
        self.get_mut(hash)?.finish_fetch().await
    }

    fn retain(&mut self, point: Arc<dyn Singular>) -> Hash {
        let hash = point.hash();
        match self.0.entry(hash) {
            btree_map::Entry::Vacant(vacant_entry) => vacant_entry.insert_entry(Retained {
                count: 0,
                point,
                fetching: None,
                resolve: None,
                ordered: false,
                waiting: Default::default(),
            }),
            btree_map::Entry::Occupied(occupied_entry) => occupied_entry,
        }
        .into_mut()
        .count += 1;
        hash
    }

    fn get_mut(&mut self, hash: Hash) -> object_rainbow::Result<&mut Retained> {
        self.0
            .get_mut(&hash)
            .ok_or_else(|| object_rainbow::error_consistency!("unknown hash"))
    }

    fn inc(&mut self, hash: Hash) -> object_rainbow::Result<()> {
        self.get_mut(hash)?.count += 1;
        Ok(())
    }

    fn dec(&mut self, hash: Hash) -> object_rainbow::Result<()> {
        let count = &mut self.get_mut(hash)?.count;
        *count -= 1;
        if *count == 0 {
            self.0.remove(&hash);
        }
        Ok(())
    }
}

pub async fn provide<E1: Send, E2: Send>(
    send: impl Send + Sink<Provide, Error = E1>,
    recv: impl Send + Stream<Item = Result<Consume, E2>>,
    publish: impl Send + Stream<Item = object_rainbow::Result<(Arc<dyn Singular>, Vec<u8>)>>,
) -> object_rainbow::Result<()>
where
    object_rainbow::Error: From<E1>,
    object_rainbow::Error: From<E2>,
{
    let mut send = pin!(send);
    let recv = recv
        .map_err(object_rainbow::Error::from)
        .map_ok(ProviderEvent::Consumed);
    let publish = publish.map_ok(ProviderEvent::Published);
    let recv = futures_util::stream::select(recv, publish);
    let mut recv = pin!(recv);
    let executor = Executor::new();
    let mut retain = Retain::default();
    executor
        .run(async {
            while let Some(event) = recv.try_next().await? {
                match event {
                    ProviderEvent::Consumed(Consume::Inc(hash)) => {
                        retain.inc(hash)?;
                    }
                    ProviderEvent::Consumed(Consume::Dec(hash)) => {
                        retain.dec(hash)?;
                    }
                    ProviderEvent::Consumed { .. } => {
                        return Err(object_rainbow::Error::Unimplemented);
                    }
                    ProviderEvent::Published((point, reason)) => {
                        let hash = retain.retain(point);
                        send.send(Provide::Publish { hash, reason }).await?;
                    }
                }
            }
            Ok(())
        })
        .await
}

enum ConsumerEvent {
    Provided(Provide),
    FetchData(Hash, oneshot::Sender<Vec<u8>>),
    Drop(Hash),
    IncChild(Hash, Address),
}

pub fn consume<E1: Send, E2: Send>(
    send: impl Send + Sink<Consume, Error = E1>,
    recv: impl Send + Stream<Item = Result<Provide, E2>>,
) -> impl Stream<Item = object_rainbow::Result<(Arc<dyn Singular>, Vec<u8>)>>
where
    object_rainbow::Error: From<E1>,
    object_rainbow::Error: From<E2>,
{
    try_stream(async move |co| {
        let (request, respond) = flume::unbounded();
        let respond = respond.into_stream().map(Ok);
        let mut send = pin!(send);
        let recv = recv
            .map_err(object_rainbow::Error::from)
            .map_ok(ConsumerEvent::Provided);
        let recv = futures_util::stream::select(recv, respond);
        let mut recv = pin!(recv);
        let mut fetches = BTreeMap::<_, Vec<oneshot::Sender<Vec<u8>>>>::new();
        while let Some(event) = recv.try_next().await? {
            match event {
                ConsumerEvent::Provided(Provide::Deliver(hash, data)) => {
                    if let Some(callbacks) = fetches.remove(&hash) {
                        for callback in callbacks {
                            callback.send(data.clone()).ok();
                        }
                    }
                }
                ConsumerEvent::Provided(Provide::Publish { hash, reason }) => {
                    let request = request.clone();
                    co.yield_((
                        Arc::new(PublishedFetch {
                            resolve: Arc::new(PublishedResolve { hash, request }),
                        }) as _,
                        reason,
                    ))
                    .await;
                }
                ConsumerEvent::FetchData(hash, callback) => {
                    match fetches.entry(hash) {
                        btree_map::Entry::Vacant(vacant_entry) => {
                            send.send(Consume::Order(hash)).await?;
                            vacant_entry.insert_entry(Vec::new())
                        }
                        btree_map::Entry::Occupied(occupied_entry) => occupied_entry,
                    }
                    .into_mut()
                    .push(callback);
                }
                ConsumerEvent::Drop(hash) => {
                    send.send(Consume::Dec(hash)).await?;
                }
                ConsumerEvent::IncChild(parent, child) => {
                    send.send(Consume::IncChild { parent, child }).await?;
                }
            }
        }
        Ok(())
    })
}

struct PublishedFetch {
    resolve: Arc<PublishedResolve>,
}

impl PublishedFetch {
    async fn fetch_raw(&self) -> object_rainbow::Result<Vec<u8>> {
        let (send, recv) = oneshot::channel();
        self.resolve
            .request
            .send_async(ConsumerEvent::FetchData(self.hash(), send))
            .await
            .map_err(|_| object_rainbow::Error::Interrupted)?;
        let data = recv.await.map_err(|_| object_rainbow::Error::Interrupted)?;
        Ok(data)
    }

    async fn make_resolve(&self) -> object_rainbow::Result<Arc<dyn Resolve>> {
        Ok(self.resolve.clone())
    }
}

impl FetchBytes for PublishedFetch {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(async move {
            let data = self.fetch_raw().await?;
            let resolve = self.make_resolve().await?;
            Ok((data, resolve))
        })
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(self.fetch_raw())
    }
}

impl Singular for PublishedFetch {
    fn hash(&self) -> Hash {
        self.resolve.hash
    }
}

struct PublishedResolve {
    hash: Hash,
    request: flume::Sender<ConsumerEvent>,
}

impl PublishedResolve {
    async fn inc_child(&self, address: Address) -> object_rainbow::Result<()> {
        self.request
            .send_async(ConsumerEvent::IncChild(self.hash, address))
            .await
            .map_err(|_| object_rainbow::Error::Interrupted)?;
        Ok(())
    }

    async fn child(&self, address: Address) -> object_rainbow::Result<PublishedFetch> {
        self.inc_child(address).await?;
        Ok(PublishedFetch {
            resolve: Arc::new(PublishedResolve {
                hash: address.hash,
                request: self.request.clone(),
            }),
        })
    }
}

impl Resolve for PublishedResolve {
    fn resolve<'a>(
        &'a self,
        address: Address,
        _: &'a Arc<dyn Resolve>,
    ) -> object_rainbow::FailFuture<'a, object_rainbow::ByteNode> {
        Box::pin(async move { self.child(address).await?.fetch_bytes().await })
    }

    fn resolve_data(&'_ self, address: Address) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(async move { self.child(address).await?.fetch_data().await })
    }
}

impl Drop for PublishedResolve {
    fn drop(&mut self) {
        self.request.try_send(ConsumerEvent::Drop(self.hash)).ok();
    }
}
