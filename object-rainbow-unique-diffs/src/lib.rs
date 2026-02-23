use object_rainbow::{
    FullHash, Inline, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    assert_impl,
};
use object_rainbow_hamt::HamtSet;
use object_rainbow_history::Forward;

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
    diffs: HamtSet,
    tree: T,
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
        &self.tree
    }
}

impl<T: Forward<D>, D: Send + FullHash> Forward<D> for UniqueDiffs<T> {
    type Output = Option<T::Output>;

    async fn forward(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        Ok(if self.diffs.insert(diff.full_hash()).await? {
            Some(self.tree.forward(diff).await?)
        } else {
            None
        })
    }
}
