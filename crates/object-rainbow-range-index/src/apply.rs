use object_rainbow::{Component, InlineOutput};
use object_rainbow_amt::AmtMap;
use object_rainbow_apply::Apply;

use crate::RangeIndex;

impl<K: Component, F: Component> Apply<AmtMap<K, (F, Option<()>)>> for RangeIndex<K>
where
    Option<F>: InlineOutput,
{
    type Output = ();

    async fn apply(
        &mut self,
        entries: AmtMap<K, (F, Option<()>)>,
    ) -> object_rainbow::Result<Self::Output> {
        self.extend(entries.to_vec().await?).await
    }
}
