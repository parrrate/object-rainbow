use std::{
    collections::{BTreeMap, btree_map},
    marker::PhantomData,
    ops::RangeBounds,
};

use bitvec::array::BitArray;
use object_rainbow::{
    Inline, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseAsInline, ParseInline, ParseInput,
    Size, Tagged, ToOutput, Topological, assert_impl,
};

type Bits = BitArray<[u8; 32]>;

#[derive(ToOutput, Tagged, ListHashes, Topological, ParseAsInline, Clone)]
pub struct ArrayMap<T> {
    bits: Bits,
    map: BTreeMap<u8, T>,
}

impl<T: InlineOutput> InlineOutput for ArrayMap<T> {}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for ArrayMap<T> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let bits = input.parse_inline::<Bits>()?;
        let map = bits
            .iter_ones()
            .map(|one| {
                Ok((
                    u8::try_from(one).expect("overflow"),
                    input.parse_inline::<T>()?,
                ))
            })
            .collect::<object_rainbow::Result<_>>()?;
        Ok(Self { bits, map })
    }
}

assert_impl!(
    impl<T, E> Inline<E> for ArrayMap<T> where T: Inline<E> {}
);

impl<T> ArrayMap<T> {
    pub fn get_mut(&mut self, key: u8) -> Option<&mut T> {
        self.map.get_mut(&key)
    }

    pub fn get(&self, key: u8) -> Option<&T> {
        self.map.get(&key)
    }

    pub fn insert(&mut self, key: u8, value: T) -> Option<T> {
        self.bits.set(key as usize, true);
        self.map.insert(key, value)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn contains(&self, key: u8) -> bool {
        self.bits[key as usize]
    }

    pub fn remove(&mut self, key: u8) -> Option<T> {
        self.bits.set(key as usize, false);
        self.map.remove(&key)
    }

    pub fn range<R: RangeBounds<u8>>(&self, range: R) -> Range<'_, T> {
        Range {
            range: self.map.range(range),
        }
    }

    pub fn pop_first(&mut self) -> Option<(u8, T)> {
        self.map
            .pop_first()
            .inspect(|&(key, _)| self.bits.set(key as usize, false))
    }

    pub const fn new() -> Self {
        Self {
            bits: BitArray {
                _ord: PhantomData,
                data: [0; _],
            },
            map: BTreeMap::new(),
        }
    }
}

pub struct Range<'a, T> {
    range: btree_map::Range<'a, u8, T>,
}

impl<'a, T> Iterator for Range<'a, T> {
    type Item = (u8, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|(&k, v)| (k, v))
    }
}

impl<T> Default for ArrayMap<T> {
    fn default() -> Self {
        Self {
            bits: Default::default(),
            map: Default::default(),
        }
    }
}

impl<T> Extend<(u8, T)> for ArrayMap<T> {
    fn extend<I: IntoIterator<Item = (u8, T)>>(&mut self, iter: I) {
        iter.into_iter().for_each(|(key, value)| {
            self.insert(key, value);
        });
    }
}

impl<T> FromIterator<(u8, T)> for ArrayMap<T> {
    fn from_iter<I: IntoIterator<Item = (u8, T)>>(iter: I) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

impl<T, const N: usize> From<[(u8, T); N]> for ArrayMap<T> {
    fn from(value: [(u8, T); N]) -> Self {
        value.into_iter().collect()
    }
}

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Size,
    MaybeHasNiche,
)]
pub struct ArraySet {
    bits: Bits,
}

assert_impl!(
    impl<E> Inline<E> for ArraySet {}
);
