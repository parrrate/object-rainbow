use object_rainbow::{Component, InlineOutput};
use object_rainbow_amt::{AmtMap, AmtSet};

use crate::Apply;

impl<K: Component, V: Component> Apply<(Option<V>, K)> for AmtMap<K, V> {
    type Output = Option<(V, K)>;

    async fn apply(
        &mut self,
        (value, key): (Option<V>, K),
    ) -> object_rainbow::Result<Self::Output> {
        if let Some(value) = value {
            self.insert_replace(key, value).await
        } else {
            self.remove_entry(&key).await
        }
        .map(|o| o.map(|(k, v)| (v, k)))
    }
}

impl<K: Component, V: Component> Apply<(V, K)> for AmtMap<K, V> {
    type Output = Option<(V, K)>;

    async fn apply(&mut self, (value, key): (V, K)) -> object_rainbow::Result<Self::Output> {
        self.insert_replace(key, value)
            .await
            .map(|o| o.map(|(k, v)| (v, k)))
    }
}

impl<K: Component, V: Component> Apply<Self> for AmtMap<K, V> {
    type Output = Self;

    async fn apply(&mut self, mut diff: Self) -> object_rainbow::Result<Self::Output> {
        self.append_swap(&mut diff).await?;
        Ok(diff)
    }
}

impl<K: Component, V: Component> Apply<AmtMap<K, Option<V>>> for AmtMap<K, V>
where
    Option<V>: InlineOutput,
{
    type Output = Self;

    async fn apply(&mut self, bulk: AmtMap<K, Option<V>>) -> object_rainbow::Result<Self::Output> {
        self.bulk(bulk).await
    }
}

impl<T: Component> Apply<(bool, T)> for AmtSet<T> {
    type Output = bool;

    async fn apply(&mut self, (remove, value): (bool, T)) -> object_rainbow::Result<Self::Output> {
        Ok(if remove {
            !self.remove(&value).await?
        } else {
            self.insert(value).await?
        })
    }
}

impl<T: Component> Apply<(Option<()>, T)> for AmtSet<T> {
    type Output = Option<T>;

    async fn apply(
        &mut self,
        (target, value): (Option<()>, T),
    ) -> object_rainbow::Result<Self::Output> {
        if target.is_some() {
            self.replace(value).await
        } else {
            self.take(&value).await
        }
    }
}

impl<T: Component> Apply<T> for AmtSet<T> {
    type Output = Option<T>;

    async fn apply(&mut self, value: T) -> object_rainbow::Result<Self::Output> {
        self.replace(value).await
    }
}

impl<T: Component> Apply<((), T)> for AmtSet<T> {
    type Output = Option<T>;

    async fn apply(&mut self, ((), value): ((), T)) -> object_rainbow::Result<Self::Output> {
        self.replace(value).await
    }
}

impl<T: Component> Apply<Self> for AmtSet<T> {
    type Output = Self;

    async fn apply(&mut self, mut diff: Self) -> object_rainbow::Result<Self::Output> {
        self.append_swap(&mut diff).await?;
        Ok(diff)
    }
}

impl<T: Component> Apply<AmtMap<T, bool>> for AmtSet<T> {
    type Output = Self;

    async fn apply(&mut self, bulk: AmtMap<T, bool>) -> object_rainbow::Result<Self::Output> {
        self.bulk(bulk).await
    }
}
