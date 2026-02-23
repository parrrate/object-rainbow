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

impl<K: ReflessObject, V: 'static + Send + Sync + Clone> Forward<(V, K)> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    type Output = Option<V>;

    fn forward(
        &mut self,
        (value, key): (V, K),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move { self.insert(&key, value).await }
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

impl<T: ReflessObject> Forward<T> for TrieSet<T> {
    type Output = bool;

    fn forward(
        &mut self,
        value: T,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move { self.insert(&value).await }
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::ParseSlice;
    use object_rainbow_trie::TrieMap;
    use smol_macros::test;

    use crate::History;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut history = History::<TrieMap<Vec<u8>, u8>, (Option<u8>, Vec<u8>)>::new();
        history.commit((Some(123), b"abc".into())).await?;
        history = history.reparse()?;
        assert_eq!(
            history.tree().await?.get(&b"abc".into()).await?.unwrap(),
            123,
        );
        Ok(())
    }
}
