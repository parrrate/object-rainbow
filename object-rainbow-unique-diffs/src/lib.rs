use object_rainbow::FullHash;
use object_rainbow_amt_set::AmtSet;
use object_rainbow_history::Forward;

pub struct UniqueDiffs<T> {
    diffs: AmtSet,
    tree: T,
}

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
