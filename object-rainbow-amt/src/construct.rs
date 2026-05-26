use object_rainbow::{InlineOutput, Traversible, length_prefixed::LpBytes, map_extra::MappedExtra};
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
    fn from_slice(prefix: Prefix, items: &mut [Item<Self>]) -> Self {
        if items.is_empty() {
            return Default::default();
        }
        if (1..items.len()).all(|i| items[0].0 == items[i].0) {
            let (_, kv) = &mut items[0];
            let (k, v) = kv.take().expect("empty kv");
            return Self::kv(prefix, k, v);
        }
        if items.iter().any(|x| x.0.is_empty()) {
            panic!("`InlineOutput` prefix extension");
        }
        let n = (0..items[0].0.len())
            .find(|&i| {
                items[1..]
                    .iter()
                    .any(|x| items[0].0[i] != *x.0.get(i).expect("`InlineOutput` prefix extension"))
            })
            .expect("`InlineOutput` prefix extension");
        let common = Vec::from(&items[0].0[..]);
        let mut counts = [0; 256];
        for x in items.iter() {
            counts[x.0[n] as usize] += 1;
        }
        let mut total = 0;
        for count in &mut counts {
            let old = *count;
            *count = total;
            total += old;
        }
        for i in 0..items.len() {
            let count = &mut counts[items[i].0[n] as usize];
            items.swap(*count, i);
            *count += 1;
        }
        let prefix = prefix.with(&*common);
        let mut children = Vec::new();
        let mut last = 0;
        for (c, x) in counts.into_iter().enumerate() {
            if x != last {
                let c = c as u8;
                children.push((c, Self::from_slice(prefix.with([c]), &mut items[last..x])));
            }
            last = x;
        }
        Self::join(common, children)
    }
}

impl<K: InlineOutput + Traversible + Clone, V: InlineOutput + Traversible + Clone> Construct
    for Node<K, V>
{
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
