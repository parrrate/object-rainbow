use object_rainbow::{Hash, InlineOutput, Traversible};
use object_rainbow_hamt::{HamtMap, HamtSet};

use crate::Apply;

impl<V: 'static + Send + Sync + Clone + Traversible + InlineOutput> Apply<(Option<V>, Hash)>
    for HamtMap<V>
{
    type Output = Option<V>;

    async fn apply(
        &mut self,
        (value, hash): (Option<V>, Hash),
    ) -> object_rainbow::Result<Self::Output> {
        if let Some(value) = value {
            self.insert(hash, value).await
        } else {
            self.remove(hash).await
        }
    }
}

impl<V: 'static + Send + Sync + Clone + Traversible + InlineOutput> Apply<(V, Hash)>
    for HamtMap<V>
{
    type Output = Option<V>;

    async fn apply(&mut self, (value, hash): (V, Hash)) -> object_rainbow::Result<Self::Output> {
        self.insert(hash, value).await
    }
}

impl Apply<(bool, Hash)> for HamtSet {
    type Output = bool;

    async fn apply(
        &mut self,
        (remove, hash): (bool, Hash),
    ) -> object_rainbow::Result<Self::Output> {
        Ok(if remove {
            !self.remove(hash).await?
        } else {
            self.insert(hash).await?
        })
    }
}

impl Apply<Hash> for HamtSet {
    type Output = bool;

    async fn apply(&mut self, hash: Hash) -> object_rainbow::Result<Self::Output> {
        self.insert(hash).await
    }
}

impl Apply<((), Hash)> for HamtSet {
    type Output = bool;

    async fn apply(&mut self, ((), hash): ((), Hash)) -> object_rainbow::Result<Self::Output> {
        self.insert(hash).await
    }
}
