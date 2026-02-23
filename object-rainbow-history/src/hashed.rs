use object_rainbow::{FullHash, Hash};

use crate::Forward;

pub struct HashedDiffs<T> {
    tree: T,
}

impl<T: Forward<Hash>, D: Send + FullHash> Forward<D> for HashedDiffs<T> {
    type Output = T::Output;

    fn forward(
        &mut self,
        diff: D,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.tree.forward(diff.full_hash())
    }
}
