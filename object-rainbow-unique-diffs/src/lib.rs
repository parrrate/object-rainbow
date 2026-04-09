use object_rainbow::{
    FullHash, Inline, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    assert_impl,
};
use object_rainbow_amt_set::AmtSet;
use object_rainbow_history::Forward;

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline)]
pub struct UniqueDiffs<T> {
    diffs: AmtSet,
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
    async fn forward(&mut self, diff: D) -> object_rainbow::Result<()> {
        if !self.diffs.contains(diff.full_hash()).await? {
            self.tree.forward(diff).await?;
        }
        Ok(())
    }
}
