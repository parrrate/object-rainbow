use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological,
};

use crate::MapDiff;

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

impl<K: Send + Clone, V: Send, M: MapToSet<K, V>> MapDiff<(Option<V>, (Option<V>, K))>
    for MappedToSet<M>
{
    type Inner = Vec<(bool, M::T)>;

    fn map(
        &self,
        (old, (new, key)): (Option<V>, (Option<V>, K)),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Inner>> {
        async move {
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
}
