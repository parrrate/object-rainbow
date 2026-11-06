use std::ops::{Bound, RangeBounds};

use async_executor::{Executor, Task};
use futures_concurrency::future::TryJoin;
use multimap::MultiMap;
use object_rainbow::{
    Component, InlineOutput, ToOutput,
    length_prefixed::LpBytes,
    pod,
    streams::{
        owp::{Owp, OwpBounds},
        owp_union::OwpUnion,
    },
};
use object_rainbow_amt::AmtMap;

#[pod(no_copy, no_ord)]
pub struct RangeIndex<K> {
    prefixes: AmtMap<LpBytes, AmtMap<K, ()>>,
    filters: AmtMap<LpBytes, AmtMap<K, ()>>,
}

type MapTask<K> = Task<Result<Vec<AmtMap<K, ()>>, object_rainbow::Error>>;

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

    fn fill_maps<'s: 'e, 'e>(
        &'s self,
        mut l: Vec<u8>,
        r: Option<Vec<u8>>,
        tasks: &mut Vec<MapTask<K>>,
        executor: &Executor<'e>,
    ) {
        if let Some(mut r) = r {
            if r <= l {
                return;
            }
            while l.len() > common_length(&l, &r) + 1 {
                let owp = self.prefixes.owp().with_bounds(
                    LpBytes(l.clone())..=LpBytes({
                        let mut r = l.clone();
                        if let Some(x) = r.last_mut() {
                            *x = 0xff;
                        }
                        r
                    }),
                );
                tasks.push(executor.spawn(async move {
                    let mut result = Vec::new();
                    owp.try_for_each_both_ends(|(_, map)| {
                        result.push(map);
                    })
                    .await?;
                    Ok(result)
                }));
                l.pop();
                while l.last().is_some_and(|x| *x == 0xff) {
                    l.pop();
                }
                let Some(x) = l.last_mut() else {
                    break;
                };
                assert!(*x < 0xff);
                *x += 1;
            }
            assert!(l <= r);
            if l == r {
                return;
            }
            while r.len() > common_length(&l, &r) + 1 {
                let x = *r.last().unwrap();
                if x != 0x00 {
                    let owp = self.prefixes.owp().with_bounds(
                        LpBytes({
                            let mut l = r.clone();
                            if let Some(x) = l.last_mut() {
                                *x = 0x00;
                            }
                            l
                        })..LpBytes(r.clone()),
                    );
                    tasks.push(executor.spawn(async move {
                        let mut result = Vec::new();
                        owp.try_for_each_both_ends(|(_, map)| {
                            result.push(map);
                        })
                        .await?;
                        Ok(result)
                    }));
                }
                r.pop();
                tasks.push(executor.spawn({
                    let r = r.clone();
                    async move {
                        let mut result = Vec::new();
                        if let Some(map) = self.filters.get(&LpBytes(r)).await? {
                            result.push(map);
                        }
                        Ok(result)
                    }
                }));
            }
            assert!(l < r);
            assert!(r.len() == common_length(&l, &r) + 1);
            if r.len() == l.len() {
                let owp = self
                    .prefixes
                    .owp()
                    .with_bounds(LpBytes(l.clone())..LpBytes(r.clone()));
                tasks.push(executor.spawn(async move {
                    let mut result = Vec::new();
                    owp.try_for_each_both_ends(|(_, map)| {
                        result.push(map);
                    })
                    .await?;
                    Ok(result)
                }));
            } else {
                assert!(r.starts_with(&l));
                assert!(r.len() == l.len() + 1);
                tasks.push(executor.spawn(async move {
                    let mut result = Vec::new();
                    if let Some(map) = self.filters.get(&LpBytes(l.clone())).await? {
                        result.push(map);
                    } else {
                        self.prefixes
                            .owp()
                            .with_bounds(
                                LpBytes({
                                    let mut l = r.clone();
                                    if let Some(x) = l.last_mut() {
                                        *x = 0x00;
                                    }
                                    l
                                })..LpBytes(r.clone()),
                            )
                            .try_for_each_increasing(|(_, map)| {
                                result.push(map);
                            })
                            .await?;
                    }
                    Ok(result)
                }));
            }
        } else {
            loop {
                let owp = self.prefixes.owp().with_bounds(
                    LpBytes(l.clone())..=LpBytes({
                        let mut r = l.clone();
                        if let Some(x) = r.last_mut() {
                            *x = 0xff;
                        }
                        r
                    }),
                );
                tasks.push(executor.spawn(async move {
                    let mut result = Vec::new();
                    owp.try_for_each_both_ends(|(_, map)| {
                        result.push(map);
                    })
                    .await?;
                    Ok(result)
                }));
                l.pop();
                while l.last().is_some_and(|x| *x == 0xff) {
                    l.pop();
                }
                let Some(x) = l.last_mut() else {
                    break;
                };
                assert!(*x < 0xff);
                *x += 1;
            }
        }
    }

    async fn vec_maps(
        &self,
        l: Bound<Vec<u8>>,
        r: Bound<Vec<u8>>,
    ) -> object_rainbow::Result<Vec<AmtMap<K, ()>>> {
        let l = match l {
            Bound::Included(l) => l,
            Bound::Excluded(mut l) => {
                while l.last().is_some_and(|x| *x == 0xff) {
                    l.pop();
                }
                let Some(x) = l.last_mut() else {
                    return Ok(Default::default());
                };
                assert!(*x < 0xff);
                *x += 1;
                l
            }
            Bound::Unbounded => Default::default(),
        };
        let r = match r {
            Bound::Included(mut r) => {
                while r.last().is_some_and(|x| *x == 0xff) {
                    r.pop();
                }
                if let Some(x) = r.last_mut() {
                    assert!(*x < 0xff);
                    *x += 1;
                    Some(r)
                } else {
                    None
                }
            }
            Bound::Excluded(r) => Some(r),
            Bound::Unbounded => None,
        };
        let mut tasks = Vec::new();
        let executor = Executor::new();
        self.fill_maps(l, r, &mut tasks, &executor);
        let result = executor
            .run(tasks.try_join())
            .await?
            .into_iter()
            .flatten()
            .collect();
        Ok(result)
    }

    async fn maps<F: ToOutput>(
        &self,
        bounds: impl RangeBounds<F>,
    ) -> object_rainbow::Result<Vec<AmtMap<K, ()>>> {
        self.vec_maps(
            bounds.start_bound().map(ToOutput::output),
            bounds.end_bound().map(ToOutput::output),
        )
        .await
    }

    pub async fn owp<F: ToOutput>(
        &self,
        bounds: impl RangeBounds<F>,
    ) -> object_rainbow::Result<impl Send + Owp<Item = (K, ())> + OwpBounds<K>>
    where
        K: Ord,
    {
        Ok(OwpUnion::new(
            self.maps(bounds).await?.into_iter().map(AmtMap::into_owp),
        ))
    }
}

fn common_length(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).take_while(|(a, b)| a == b).count()
}
