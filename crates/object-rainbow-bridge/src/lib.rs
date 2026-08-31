use std::{pin::pin, sync::Arc};

use futures_util::{Sink, Stream, TryStreamExt};
use genawaiter_try_stream::try_stream;
use object_rainbow::{Address, Hash, Singular};

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
    Deliver(Vec<u8>),
    /// Push a reference towards the client and increase server-side refcount by 1.
    Publish { hash: Hash, reason: Vec<u8> },
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
        let _ = pin!(send);
        let mut recv = pin!(recv);
        let _ = co;
        while let Some(provided) = recv.try_next().await? {
            match provided {
                Provide::Deliver { .. } => {}
                Provide::Publish { .. } => return Err(object_rainbow::Error::Unimplemented),
            }
        }
        Err(object_rainbow::Error::Unimplemented)
    })
}
