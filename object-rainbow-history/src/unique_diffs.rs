use object_rainbow::{
    FullHash, Inline, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    assert_impl,
};
use object_rainbow_hamt::HamtSet;

use crate::{Forward, Sequential, hashed::HashedDiffs, skip::SkipDiffs};

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    PartialEq,
    Eq,
    Default,
)]
pub struct UniqueDiffs<T> {
    inner: Sequential<HashedDiffs<HamtSet>, SkipDiffs<T>>,
}

assert_impl!(
    impl<T, E> Inline<E> for UniqueDiffs<T>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
    {
    }
);

impl<T> UniqueDiffs<T> {
    pub fn tree(&self) -> &T {
        &self.inner.second().0
    }
}

impl<T: Forward<D>, D: Send + FullHash + Clone> Forward<D> for UniqueDiffs<T> {
    type Output = Option<T::Output>;

    async fn forward(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        self.inner.forward(diff).await
    }
}
