use object_rainbow::{Hash, InlineOutput, Traversible};
use object_rainbow_hamt::{HamtMap, HamtSet};

use crate::Apply;

impl<V: 'static + Send + Sync + Clone + Traversible + InlineOutput> Apply<(V, Hash)>
    for HamtMap<V>
{
    type Output = Option<V>;

    fn apply(
        &mut self,
        (value, hash): (V, Hash),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.insert(hash, value)
    }
}

impl Apply<Hash> for HamtSet {
    type Output = bool;

    fn apply(
        &mut self,
        hash: Hash,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.insert(hash)
    }
}
