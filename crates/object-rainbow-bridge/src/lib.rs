use std::{
    collections::{BTreeMap, btree_map},
    pin::pin,
    sync::Arc,
};

use futures_channel::oneshot;
use futures_util::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use genawaiter_try_stream::try_stream;
use object_rainbow::{Address, FetchBytes, Hash, Resolve, Singular};

/// Commands coming from a consumer.
pub enum Consume {
    /// Request [`Provide::Deliver`]y. Requires refcount of at least 1.
    Order(Hash),
    /// Increase server-side refcount by 1.
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

enum ConsumerEvent {
    Provided(Provide),
    FetchData(Hash, oneshot::Sender<Vec<u8>>),
    #[expect(unused)]
    MakeResolve(Hash, oneshot::Sender<Arc<dyn Resolve>>),
    Drop(Hash),
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
                    co.yield_((Arc::new(PublishedFetch { hash, request }) as _, reason))
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
                    return Err(object_rainbow::Error::Unimplemented);
                }
                ConsumerEvent::MakeResolve { .. } => {
                    return Err(object_rainbow::Error::Unimplemented);
                }
                ConsumerEvent::Drop(hash) => {
                    send.send(Consume::Dec(hash)).await?;
                }
            }
        }
        Ok(())
    })
}

struct PublishedFetch {
    hash: Hash,
    request: flume::Sender<ConsumerEvent>,
}

impl FetchBytes for PublishedFetch {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(core::future::ready(Err(
            object_rainbow::Error::Unimplemented,
        )))
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let (send, recv) = oneshot::channel();
            self.request
                .send_async(ConsumerEvent::FetchData(self.hash, send))
                .await
                .map_err(|_| object_rainbow::Error::Interrupted)?;
            let data = recv.await.map_err(|_| object_rainbow::Error::Interrupted)?;
            Ok(data)
        })
    }
}

impl Singular for PublishedFetch {
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl Drop for PublishedFetch {
    fn drop(&mut self) {
        self.request.try_send(ConsumerEvent::Drop(self.hash)).ok();
    }
}
