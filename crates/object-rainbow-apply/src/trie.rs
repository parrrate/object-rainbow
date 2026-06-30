use object_rainbow::{InlineOutput, ReflessObject, Traversible};
use object_rainbow_trie::{TrieMap, TrieSet};

use crate::Apply;

impl<K: ReflessObject, V: 'static + Send + Sync + Clone> Apply<(Option<V>, K)> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    type Output = Option<V>;

    async fn apply(
        &mut self,
        (value, key): (Option<V>, K),
    ) -> object_rainbow::Result<Self::Output> {
        if let Some(value) = value {
            self.insert(&key, value).await
        } else {
            self.remove(&key).await
        }
    }
}

impl<K: ReflessObject, V: 'static + Send + Sync + Clone> Apply<(V, K)> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    type Output = Option<V>;

    async fn apply(&mut self, (value, key): (V, K)) -> object_rainbow::Result<Self::Output> {
        self.insert(&key, value).await
    }
}

/// `true` represents removal, `false` represents insertion to keep layout equivalence.
impl<T: ReflessObject> Apply<(bool, T)> for TrieSet<T> {
    type Output = bool;

    async fn apply(&mut self, (remove, value): (bool, T)) -> object_rainbow::Result<Self::Output> {
        Ok(if remove {
            !self.remove(&value).await?
        } else {
            self.insert(&value).await?
        })
    }
}

impl<T: ReflessObject> Apply<T> for TrieSet<T> {
    type Output = bool;

    async fn apply(&mut self, value: T) -> object_rainbow::Result<Self::Output> {
        self.insert(&value).await
    }
}

impl<T: ReflessObject> Apply<((), T)> for TrieSet<T> {
    type Output = bool;

    async fn apply(&mut self, ((), value): ((), T)) -> object_rainbow::Result<Self::Output> {
        self.insert(&value).await
    }
}
