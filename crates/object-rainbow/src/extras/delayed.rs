use std::marker::PhantomData;

use crate::{Fetch, FetchBytes, ParseSliceExtra, Singular};

pub struct DelayedExtra<E, F, T>(pub E, pub F, pub PhantomData<T>);

impl<E, F: FetchBytes, T> FetchBytes for DelayedExtra<E, F, T> {
    fn fetch_bytes(&'_ self) -> crate::FailFuture<'_, crate::ByteNode> {
        self.1.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> crate::FailFuture<'_, Vec<u8>> {
        self.1.fetch_data()
    }
}

impl<E: Send + Sync, F: Singular, T: Send + Sync> Singular for DelayedExtra<E, F, T> {
    fn hash(&self) -> crate::Hash {
        self.1.hash()
    }
}

impl<
    E: Send + Sync + Fetch<T: Send + Sync + Clone>,
    F: 'static + Send + Sync + FetchBytes,
    T: Send + Sync + ParseSliceExtra<E::T>,
> DelayedExtra<E, F, T>
{
    async fn fetch_node(&self) -> object_rainbow::Result<crate::Node<T>> {
        let (data, resolve) = self.fetch_bytes().await?;
        let object = T::parse_slice_extra(&data, &resolve, &self.0.fetch().await?)?;
        Ok((object, resolve))
    }
}

impl<
    E: Send + Sync + Fetch<T: Send + Sync + Clone>,
    F: 'static + Send + Sync + FetchBytes,
    T: Send + Sync + ParseSliceExtra<E::T>,
> Fetch for DelayedExtra<E, F, T>
{
    type T = T;

    fn fetch_full(&'_ self) -> crate::FailFuture<'_, crate::Node<Self::T>> {
        Box::pin(self.fetch_node())
    }

    fn fetch(&'_ self) -> crate::FailFuture<'_, Self::T> {
        Box::pin(async move { Ok(self.fetch_node().await?.0) })
    }
}
