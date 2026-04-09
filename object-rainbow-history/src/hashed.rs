use object_rainbow::{
    FullHash, Hash, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged,
    ToOutput, Topological,
};

use crate::Apply;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct HashedDiffs<T> {
    tree: T,
}

impl<T> HashedDiffs<T> {
    pub fn tree(&self) -> &T {
        &self.tree
    }
}

impl<T: Apply<Hash>, D: Send + FullHash> Apply<D> for HashedDiffs<T> {
    type Output = T::Output;

    fn apply(
        &mut self,
        diff: D,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.tree.apply(diff.full_hash())
    }
}
