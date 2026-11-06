use futures_concurrency::future::TryJoin;
use multimap::MultiMap;
use object_rainbow::{Component, InlineOutput, length_prefixed::LpBytes, pod};
use object_rainbow_amt::AmtMap;

#[pod(no_copy, no_ord)]
pub struct RangeIndex<K> {
    prefixes: AmtMap<LpBytes, AmtMap<K, ()>>,
    filters: AmtMap<LpBytes, AmtMap<K, ()>>,
}

impl<K: Component> RangeIndex<K> {
    pub async fn extend<F: Component>(
        &mut self,
        entries: impl IntoIterator<Item = (K, F)>,
    ) -> object_rainbow::Result<()>
    where
        Option<F>: InlineOutput,
    {
        let entries = entries
            .into_iter()
            .map(|(k, f)| (f.vec(), k))
            .collect::<Vec<_>>();
        let f1 = self.prefixes.bulk2(
            entries
                .iter()
                .flat_map(|(filter, k)| {
                    (0..=filter.len())
                        .map(|n| filter[..n].to_vec())
                        .map(|key| (LpBytes(key), (k.clone(), ())))
                })
                .collect::<MultiMap<_, _>>()
                .into_iter()
                .map(|(k, kv)| {
                    (
                        k,
                        kv.into_iter()
                            .map(|(k, v)| (k, Some(v)))
                            .collect::<AmtMap<_, _>>(),
                    )
                })
                .collect::<AmtMap<_, _>>(),
        );
        let entries = entries
            .into_iter()
            .map(|(f, k)| {
                (
                    f,
                    [(k.clone(), Some(()))]
                        .into_iter()
                        .collect::<AmtMap<K, _>>(),
                )
            })
            .collect::<Vec<_>>();
        let f2 = self.filters.bulk2(
            entries
                .into_iter()
                .map(|(filter, map)| (LpBytes(filter), map))
                .collect::<AmtMap<_, _>>(),
        );
        (f1, f2).try_join().await?;
        Ok(())
    }
}
