use object_rainbow::{InlineOutput, ReflessObject, Traversible};
use object_rainbow_trie::TrieMap;

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
