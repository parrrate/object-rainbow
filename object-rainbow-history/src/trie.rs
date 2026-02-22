use object_rainbow::{InlineOutput, ReflessObject, Traversible};
use object_rainbow_trie::{TrieMap, TrieSet};

use crate::Forward;

impl<K: ReflessObject, V: 'static + Send + Sync + Clone> Forward<(K, Option<V>)> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    fn forward(
        &mut self,
        (key, value): (K, Option<V>),
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        async move {
            if let Some(value) = value {
                self.insert(&key, value).await?;
            } else {
                self.remove(&key).await?;
            }
            Ok(())
        }
    }
}

/// `true` represents removal, `false` represents insertion to keep layout equivalence.
impl<T: ReflessObject> Forward<(T, bool)> for TrieSet<T> {
    fn forward(
        &mut self,
        (value, remove): (T, bool),
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        async move {
            if remove {
                self.remove(&value).await?;
            } else {
                self.insert(&value).await?;
            }
            Ok(())
        }
    }
}
