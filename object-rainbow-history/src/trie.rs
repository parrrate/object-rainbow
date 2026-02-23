use object_rainbow::{InlineOutput, ReflessObject, Traversible};
use object_rainbow_trie::{TrieMap, TrieSet};

use crate::Forward;

impl<K: ReflessObject, V: 'static + Send + Sync + Clone> Forward<(Option<V>, K)> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    type Output = Option<V>;

    fn forward(
        &mut self,
        (value, key): (Option<V>, K),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move {
            if let Some(value) = value {
                self.insert(&key, value).await
            } else {
                self.remove(&key).await
            }
        }
    }
}

/// `true` represents removal, `false` represents insertion to keep layout equivalence.
impl<T: ReflessObject> Forward<(bool, T)> for TrieSet<T> {
    type Output = bool;

    fn forward(
        &mut self,
        (remove, value): (bool, T),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move {
            Ok(if remove {
                !self.remove(&value).await?
            } else {
                self.insert(&value).await?
            })
        }
    }
}
