use std::{pin::pin, sync::Arc};

use futures_channel::oneshot;
use futures_util::{Sink, Stream, StreamExt, TryStreamExt};
use genawaiter_try_stream::try_stream;
use object_rainbow::{Address, FetchBytes, Hash, Singular};

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
    #[expect(unused)]
    Fetch(Hash, oneshot::Sender<Vec<u8>>),
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
        let (request, respond) = flume::bounded(0);
        let respond = respond.into_stream().map(Ok);
        let _ = pin!(send);
        let recv = recv
            .map_err(object_rainbow::Error::from)
            .map_ok(ConsumerEvent::Provided);
        let recv = futures_util::stream::select(recv, respond);
        let mut recv = pin!(recv);
        let _ = co;
        while let Some(provided) = recv.try_next().await? {
            match provided {
                ConsumerEvent::Provided(Provide::Deliver { .. }) => {}
                ConsumerEvent::Provided(Provide::Publish { hash, reason }) => {
                    let request = request.clone();
                    co.yield_((Arc::new(PublishedFetch { hash, request }) as _, reason))
                        .await;
                }
                ConsumerEvent::Fetch { .. } => return Err(object_rainbow::Error::Unimplemented),
            }
        }
        Ok(())
    })
}

struct PublishedFetch {
    hash: Hash,
    #[expect(unused)]
    request: flume::Sender<ConsumerEvent>,
}

impl FetchBytes for PublishedFetch {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(core::future::ready(Err(
            object_rainbow::Error::Unimplemented,
        )))
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(core::future::ready(Err(
            object_rainbow::Error::Unimplemented,
        )))
    }
}

impl Singular for PublishedFetch {
    fn hash(&self) -> Hash {
        self.hash
    }
}
