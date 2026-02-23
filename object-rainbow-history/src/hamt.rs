use object_rainbow::{Hash, InlineOutput, Traversible};
use object_rainbow_hamt::{HamtMap, HamtSet};

use crate::Forward;

impl<V: 'static + Send + Sync + Clone + Traversible + InlineOutput> Forward<(V, Hash)>
    for HamtMap<V>
{
    type Output = Option<V>;

    fn forward(
        &mut self,
        (value, hash): (V, Hash),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.insert(hash, value)
    }
}

impl Forward<Hash> for HamtSet {
    type Output = bool;

    fn forward(
        &mut self,
        hash: Hash,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.insert(hash)
    }
}
