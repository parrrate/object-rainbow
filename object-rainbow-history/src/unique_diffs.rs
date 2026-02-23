use object_rainbow::{
    FullHash, Inline, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    assert_impl,
};
use object_rainbow_hamt::HamtSet;

use crate::{Apply, Sequential, hashed::HashedDiffs, skip::FilterDiffs};

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
    inner: Sequential<HashedDiffs<HamtSet>, FilterDiffs<T>>,
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

impl<T: Apply<D>, D: Send + FullHash + Clone> Apply<D> for UniqueDiffs<T> {
    type Output = Option<T::Output>;

    async fn apply(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        self.inner.apply(diff).await
    }
}
