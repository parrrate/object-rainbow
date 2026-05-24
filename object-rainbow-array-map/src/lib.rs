use std::{
    collections::{BTreeMap, btree_map},
    marker::PhantomData,
    ops::{Deref, DerefMut, RangeBounds},
};

use bitvec::array::BitArray;
use object_rainbow::{
    Equivalent, Inline, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseAsInline, ParseInline,
    ParseInput, PointInput, RainbowIterator, Size, Tagged, ToOutput, Topological, assert_impl,
};

type Bits = BitArray<[u8; 32]>;

#[derive(Tagged, ListHashes, Topological, ParseAsInline, Clone, PartialEq, Eq)]
pub struct ArrayMap<T> {
    bits: Bits,
    map: BTreeMap<u8, T>,
}

impl<T: InlineOutput> ToOutput for ArrayMap<T> {
    fn to_output(&self, output: &mut impl object_rainbow::Output) {
        self.bits.to_output(output);
        self.map.values().iter_to_output(output);
    }
}

impl<T: InlineOutput> InlineOutput for ArrayMap<T> {}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for ArrayMap<T> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let bits = input.parse_inline::<Bits>()?;
        let map = bits
            .iter_ones()
            .map(|one| Ok((u8::try_from(one).expect("overflow"), input.parse_inline()?)))
            .collect::<object_rainbow::Result<_>>()?;
        Ok(Self { bits, map })
    }
}

assert_impl!(
    impl<T, E> Inline<E> for ArrayMap<T>
    where
        T: Inline<E>,
        E: Clone,
    {
    }
);

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, ParseAsInline, Clone)]
pub struct KeyedArrayMap<T>(pub ArrayMap<T>);

impl<T: ParseInline<I::WithExtra<(u8, I::Extra)>>, I: PointInput> ParseInline<I>
    for KeyedArrayMap<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let bits = input.parse_inline::<Bits>()?;
        let map = bits
            .iter_ones()
            .map(|one| {
                let key = u8::try_from(one).expect("overflow");
                Ok((key, input.parse_inline_extra((key, input.extra().clone()))?))
            })
            .collect::<object_rainbow::Result<_>>()?;
        Ok(Self(ArrayMap { bits, map }))
    }
}

assert_impl!(
    impl<T, E> Inline<E> for KeyedArrayMap<T>
    where
        T: Inline<(u8, E)>,
        E: 'static + Clone,
    {
    }
);

impl<T> Deref for KeyedArrayMap<T> {
    type Target = ArrayMap<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for KeyedArrayMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for KeyedArrayMap<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

    pub fn iter(&self) -> Entries<'_, T> {
        Entries {
            inner: self.map.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> EntriesMut<'_, T> {
        EntriesMut {
            inner: self.map.iter_mut(),
        }
    }

    pub fn append(&mut self, other: &mut Self) {
        while let Some((key, value)) = other.pop_first() {
            self.insert(key, value);
        }
    }

    pub fn retain(&mut self, mut f: impl FnMut(u8, &mut T) -> bool) {
        let mut remove = Vec::new();
        for (key, value) in self.iter_mut() {
            if !f(key, value) {
                remove.push(key);
            }
        }
        for key in remove {
            self.remove(key);
        }
    }

    pub fn entry(&mut self, key: u8) -> Entry<'_, T> {
        let map = self;
        if map.contains(key) {
            Entry::Occupied(OccupiedEntry { map, key })
        } else {
            Entry::Vacant(VacantEntry { map, key })
        }
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ArrayMap<U> {
        let Self { bits, map } = self;
        let map = map.into_iter().map(|(k, v)| (k, f(v))).collect();
        ArrayMap { bits, map }
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

pub struct IntoEntries<T> {
    inner: std::collections::btree_map::IntoIter<u8, T>,
}

impl<T> Iterator for IntoEntries<T> {
    type Item = (u8, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> IntoIterator for ArrayMap<T> {
    type Item = (u8, T);

    type IntoIter = IntoEntries<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoEntries {
            inner: self.map.into_iter(),
        }
    }
}

pub struct Entries<'a, T> {
    inner: std::collections::btree_map::Iter<'a, u8, T>,
}

impl<'a, T> Iterator for Entries<'a, T> {
    type Item = (u8, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.inner.next()?;
        Some((*key, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a ArrayMap<T> {
    type Item = (u8, &'a T);

    type IntoIter = Entries<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct EntriesMut<'a, T> {
    inner: std::collections::btree_map::IterMut<'a, u8, T>,
}

impl<'a, T> Iterator for EntriesMut<'a, T> {
    type Item = (u8, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.inner.next()?;
        Some((*key, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a mut ArrayMap<T> {
    type Item = (u8, &'a mut T);

    type IntoIter = EntriesMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub enum Entry<'a, T> {
    Vacant(VacantEntry<'a, T>),
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T> Entry<'a, T> {
    pub fn and_modify(mut self, f: impl FnOnce(&mut T)) -> Self {
        if let Self::Occupied(e) = &mut self {
            f(e.get_mut());
        }
        self
    }

    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        match self {
            Self::Vacant(e) => e.insert_entry(value),
            Self::Occupied(mut e) => {
                e.insert(value);
                e
            }
        }
    }

    pub fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        self.or_insert_with(Default::default)
    }

    pub fn or_insert(self, default: T) -> &'a mut T {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with(self, default: impl FnOnce() -> T) -> &'a mut T {
        match self {
            Self::Vacant(e) => e.insert(default()),
            Self::Occupied(e) => e.into_mut(),
        }
    }
}

pub struct VacantEntry<'a, T> {
    map: &'a mut ArrayMap<T>,
    key: u8,
}

impl<'a, T> VacantEntry<'a, T> {
    pub fn insert(self, value: T) -> &'a mut T {
        self.insert_entry(value).into_mut()
    }

    pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, T> {
        assert!(self.map.insert(self.key, value).is_none());
        OccupiedEntry {
            map: self.map,
            key: self.key,
        }
    }
}

pub struct OccupiedEntry<'a, T> {
    map: &'a mut ArrayMap<T>,
    key: u8,
}

impl<'a, T> OccupiedEntry<'a, T> {
    pub fn get(&self) -> &T {
        self.map.get(self.key).expect("occupied")
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.map.get_mut(self.key).expect("occupied")
    }

    pub fn insert(&mut self, value: T) -> T {
        std::mem::replace(self.get_mut(), value)
    }

    pub fn into_mut(self) -> &'a mut T {
        self.map.get_mut(self.key).expect("occupied")
    }

    pub fn remove(self) -> T {
        self.map.remove(self.key).expect("occupied")
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
    Default,
)]
pub struct ArraySet {
    bits: Bits,
}

assert_impl!(
    impl<E> Inline<E> for ArraySet where E: Clone {}
);

impl ArraySet {
    pub fn insert(&mut self, key: u8) -> bool {
        if !self.bits[key as usize] {
            self.bits.set(key as usize, true);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, key: u8) -> bool {
        self.bits[key as usize]
    }
}

impl Extend<u8> for ArraySet {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        iter.into_iter().for_each(|key| {
            self.insert(key);
        });
    }
}

impl FromIterator<u8> for ArraySet {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut set = Self::default();
        set.extend(iter);
        set
    }
}

impl<const N: usize> From<[u8; N]> for ArraySet {
    fn from(value: [u8; N]) -> Self {
        value.into_iter().collect()
    }
}

impl Equivalent<ArrayMap<()>> for ArraySet {
    fn into_equivalent(self) -> ArrayMap<()> {
        ArrayMap {
            bits: self.bits,
            map: self
                .bits
                .iter_ones()
                .map(|one| (u8::try_from(one).expect("overflow"), ()))
                .collect(),
        }
    }

    fn from_equivalent(ArrayMap { bits, .. }: ArrayMap<()>) -> Self {
        Self { bits }
    }
}

#[test]
fn reparse() -> object_rainbow::Result<()> {
    use object_rainbow::ParseSlice;
    let mut map = ArrayMap::<u8>::new();
    map.insert(12, 34);
    map = map.reparse()?;
    assert_eq!(map.get(12), Some(&34));
    Ok(())
}
