use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, derive_for_wrapped,
};

use crate::Apply;

#[derive_for_wrapped]
pub trait MapToSet<K: Send, V: Send>: Send + Sync {
    type T: Send;
    fn map(&self, key: K, value: V)
    -> impl Send + Future<Output = object_rainbow::Result<Self::T>>;
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct MappedToSet<M>(M);

impl<K: Send + Clone, V: Send, M: MapToSet<K, V>> Apply<(Option<V>, (Option<V>, K))>
    for MappedToSet<M>
{
    type Output = Vec<(bool, M::T)>;

    async fn apply(
        &mut self,
        (old, (new, key)): (Option<V>, (Option<V>, K)),
    ) -> object_rainbow::Result<Self::Output> {
        let mut diff = Vec::new();
        if let Some(value) = old {
            diff.push((true, self.0.map(key.clone(), value).await?));
        }
        if let Some(value) = new {
            diff.push((false, self.0.map(key, value).await?));
        }
        Ok(diff)
    }
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct ToSet;

impl<K: Send, V: Send> MapToSet<K, V> for ToSet {
    type T = (K, V);

    fn map(
        &self,
        key: K,
        value: V,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        core::future::ready(Ok((key, value)))
    }
}
