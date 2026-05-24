use object_rainbow::{InlineOutput, Traversible};
use object_rainbow_amt::{AmtMap, AmtSet};

use crate::Apply;

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone>
    Apply<(Option<V>, K)> for AmtMap<K, V>
{
    type Output = Option<V>;

    async fn apply(
        &mut self,
        (value, key): (Option<V>, K),
    ) -> object_rainbow::Result<Self::Output> {
        if let Some(value) = value {
            self.insert(key, value).await
        } else {
            self.remove(&key).await
        }
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Apply<(V, K)>
    for AmtMap<K, V>
{
    type Output = Option<V>;

    async fn apply(&mut self, (value, key): (V, K)) -> object_rainbow::Result<Self::Output> {
        self.insert(key, value).await
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Apply<Self>
    for AmtMap<K, V>
{
    type Output = Self;

    async fn apply(&mut self, mut diff: Self) -> object_rainbow::Result<Self::Output> {
        self.append_swap(&mut diff).await?;
        Ok(diff)
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone>
    Apply<AmtMap<K, Option<V>>> for AmtMap<K, V>
where
    Option<V>: InlineOutput,
{
    type Output = Self;

    async fn apply(&mut self, bulk: AmtMap<K, Option<V>>) -> object_rainbow::Result<Self::Output> {
        self.bulk(bulk).await
    }
}

impl<T: InlineOutput + Traversible + Clone> Apply<(bool, T)> for AmtSet<T> {
    type Output = bool;

    async fn apply(&mut self, (remove, value): (bool, T)) -> object_rainbow::Result<Self::Output> {
        Ok(if remove {
            !self.remove(&value).await?
        } else {
            self.insert(value).await?
        })
    }
}

impl<T: InlineOutput + Traversible + Clone> Apply<T> for AmtSet<T> {
    type Output = bool;

    async fn apply(&mut self, value: T) -> object_rainbow::Result<Self::Output> {
        self.insert(value).await
    }
}

impl<T: InlineOutput + Traversible + Clone> Apply<((), T)> for AmtSet<T> {
    type Output = bool;

    async fn apply(&mut self, ((), value): ((), T)) -> object_rainbow::Result<Self::Output> {
        self.insert(value).await
    }
}

impl<T: InlineOutput + Traversible + Clone> Apply<Self> for AmtSet<T> {
    type Output = Self;

    async fn apply(&mut self, mut diff: Self) -> object_rainbow::Result<Self::Output> {
        self.append_swap(&mut diff).await?;
        Ok(diff)
    }
}

impl<T: InlineOutput + Traversible + Clone> Apply<AmtMap<T, bool>> for AmtSet<T> {
    type Output = Self;

    async fn apply(&mut self, bulk: AmtMap<T, bool>) -> object_rainbow::Result<Self::Output> {
        self.bulk(bulk).await
    }
}
