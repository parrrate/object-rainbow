use object_rainbow::{InlineOutput, Traversible};
use object_rainbow_amt::AmtMap;

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
