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
        assert_eq!(history.tree().await?.get(&b"abc".into()).await?, Some(123));
        Ok(())
    }
}
