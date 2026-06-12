use object_rainbow::{Component, length_prefixed::LpBytes, map_extra::MappedExtra};
use object_rainbow_array_map::KeyedArrayMap;
use object_rainbow_parse_prefix::{Prefix, WithBytes, WithPrefix};
use object_rainbow_point::IntoPoint;

use crate::Node;

type Item<C> = (Vec<u8>, Option<(<C as Construct>::K, <C as Construct>::V)>);

pub(crate) trait Construct: Default {
    type K;
    type V;
    fn kv(prefix: Prefix, k: Self::K, v: Self::V) -> Self;
    fn join(branch: Vec<u8>, children: Vec<(u8, Self)>) -> Self;
    fn from_slice(m: usize, prefix: Prefix, items: &mut [Item<Self>]) -> Self {
        if items.is_empty() {
            return Default::default();
        }
        let Some(n) = (m..items
            .iter()
            .map(|x| x.0.len())
            .min()
            .expect("at least one item"))
            .find(|&i| {
                items[1..]
                    .iter()
                    .any(|x| items[0].0[i] != *x.0.get(i).expect("`InlineOutput` prefix extension"))
            })
        else {
            let (_, kv) = &mut items[0];
            let (k, v) = kv.take().expect("empty kv");
            return Self::kv(prefix, k, v);
        };
        let common = Vec::from(&items[0].0[m..n]);
        let mut counts = [0; 256];
        for x in items.iter() {
            counts[x.0[n] as usize] += 1;
        }
        let mut ends = [0; 256];
        let mut total = 0;
        for (count, end) in counts.iter_mut().zip(&mut ends) {
            let old = *count;
            *count = total;
            total += old;
            *end = total;
        }
        for i in 0..items.len() {
            loop {
                let c = items[i].0[n] as usize;
                let count = &mut counts[c];
                if *count == ends[c] {
                    break;
                }
                assert!(*count < items.len());
                if *count == i {
                    *count += 1;
                    break;
                }
                items.swap(*count, i);
                *count += 1;
            }
        }
        assert!(items.is_sorted_by_key(|item| item.0[n]));
        let prefix = prefix.with(&*common);
        let mut children = Vec::new();
        let mut last = 0;
        for (c, x) in counts.into_iter().enumerate() {
            if x != last {
                let c = c as u8;
                children.push((
                    c,
                    Self::from_slice(n + 1, prefix.with([c]), &mut items[last..x]),
                ));
            }
            last = x;
        }
        Self::join(common, children)
    }
}

impl<K: Component, V: Component> Construct for Node<K, V> {
    type K = K;
    type V = V;

    fn kv(prefix: Prefix, k: Self::K, v: Self::V) -> Self {
        Self::Leaf(
            WithPrefix::new(prefix, k).expect("must be correct by construction"),
            MappedExtra(Default::default(), v),
        )
    }

    fn join(branch: Vec<u8>, children: Vec<(u8, Self)>) -> Self {
        Self::Sub(
            MappedExtra(
                WithBytes(LpBytes(branch)),
                KeyedArrayMap(
                    children
                        .into_iter()
                        .map(|(k, v)| (k, MappedExtra(Default::default(), v)))
                        .collect(),
                ),
            )
            .point(),
        )
    }
}
