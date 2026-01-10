use std::{
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
    pin::pin,
};

use futures_util::{Stream, TryStream, TryStreamExt};
use genawaiter_try_stream::{Co, try_stream};
use object_rainbow::{
    Fetch, InlineOutput, ListHashes, ObjectMarker, Parse, ParseSliceRefless, ReflessObject, Tagged,
    ToOutput, Topological, Traversible, length_prefixed::LpBytes,
};
use object_rainbow_point::{IntoPoint, Point};

#[cfg(feature = "serde")]
mod serde;

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
#[topology(recursive)]
pub struct Trie<T> {
    value: Option<T>,
    #[tags(skip)]
    children: BTreeMap<u8, Point<(LpBytes, Self)>>,
}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            children: Default::default(),
        }
    }
}

trait TrieNode: Sized {
    fn c_get_mut(&mut self, key: u8) -> Option<&mut Point<(LpBytes, Self)>>;
    fn c_get(&self, key: u8) -> Option<&Point<(LpBytes, Self)>>;
    fn c_insert(&mut self, key: u8, point: Point<(LpBytes, Self)>);
    fn c_empty(&self) -> bool;
    fn c_len(&self) -> usize;
    fn c_remove(&mut self, key: u8);
    fn c_range(
        &self,
        min_inclusive: u8,
        max_inclusive: u8,
    ) -> impl Iterator<Item = (&u8, &Point<(LpBytes, Self)>)>;
    fn c_pop_first(&mut self) -> Option<(u8, Point<(LpBytes, Self)>)>;
}

impl<T> TrieNode for Trie<T> {
    fn c_get_mut(&mut self, key: u8) -> Option<&mut Point<(LpBytes, Self)>> {
        self.children.get_mut(&key)
    }

    fn c_get(&self, key: u8) -> Option<&Point<(LpBytes, Self)>> {
        self.children.get(&key)
    }

    fn c_insert(&mut self, key: u8, point: Point<(LpBytes, Self)>) {
        self.children.insert(key, point);
    }

    fn c_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn c_len(&self) -> usize {
        self.children.len()
    }

    fn c_remove(&mut self, key: u8) {
        self.children.remove(&key);
    }

    fn c_range(
        &self,
        min_inclusive: u8,
        max_inclusive: u8,
    ) -> impl Iterator<Item = (&u8, &Point<(LpBytes, Self)>)> {
        self.children.range(min_inclusive..=max_inclusive)
    }

    fn c_pop_first(&mut self) -> Option<(u8, Point<(LpBytes, Self)>)> {
        self.children.pop_first()
    }
}

impl<T> Trie<T> {
    fn from_value(value: T) -> Self {
        Self {
            value: Some(value),
            children: Default::default(),
        }
    }
}

impl<T: 'static + Send + Sync + Clone> Trie<T>
where
    Option<T>: Traversible + InlineOutput,
{
    pub async fn get(&self, key: &[u8]) -> object_rainbow::Result<Option<T>> {
        let Some((first, key)) = key.split_first() else {
            return Ok(self.value.clone());
        };
        let Some(point) = self.c_get(*first) else {
            return Ok(None);
        };
        let (prefix, trie) = point.fetch().await?;
        let Some(key) = key.strip_prefix(prefix.as_slice()) else {
            return Ok(None);
        };
        Box::pin(trie.get(key)).await
    }

    pub async fn insert(&mut self, key: &[u8], value: T) -> object_rainbow::Result<Option<T>> {
        let Some((first, key)) = key.split_first() else {
            return Ok(self.value.replace(value));
        };
        let Some(point) = self.c_get_mut(*first) else {
            self.c_insert(
                *first,
                (LpBytes(key.into()), Self::from_value(value)).point(),
            );
            return Ok(None);
        };
        let (prefix, trie) = &mut *point.fetch_mut().await?;
        if let Some(key) = key.strip_prefix(prefix.as_slice()) {
            return Box::pin(trie.insert(key, value)).await;
        }
        if let Some(suffix) = prefix.strip_prefix(key) {
            let child = std::mem::replace(trie, Self::from_value(value));
            let (first, suffix) = suffix.split_first().expect("must be at least 1");
            trie.c_insert(*first, (LpBytes(suffix.into()), child).point());
            prefix.0.truncate(key.len());
        } else {
            let common = prefix.iter().zip(key).take_while(|(a, b)| a == b).count();
            let child = std::mem::take(trie);
            trie.c_insert(
                prefix[common],
                (LpBytes(prefix[common + 1..].to_vec()), child).point(),
            );
            trie.c_insert(
                key[common],
                (
                    LpBytes(prefix[common + 1..].to_vec()),
                    Self::from_value(value),
                )
                    .point(),
            );
            prefix.0.truncate(common);
        }
        Ok(None)
    }

    pub fn is_empty(&self) -> bool {
        self.c_empty() && self.value.is_none()
    }

    pub async fn remove(&mut self, key: &[u8]) -> object_rainbow::Result<Option<T>> {
        let Some((first, key)) = key.split_first() else {
            return Ok(self.value.take());
        };
        let (item, is_empty) = {
            let Some(point) = self.c_get_mut(*first) else {
                return Ok(None);
            };
            let (prefix, trie) = &mut *point.fetch_mut().await?;
            let Some(key) = key.strip_prefix(prefix.as_slice()) else {
                return Ok(None);
            };
            let item = Box::pin(trie.remove(key)).await?;
            if trie.value.is_none()
                && trie.c_len() < 2
                && let Some((first, point)) = trie.c_pop_first()
            {
                let (suffix, child) = point.fetch().await?;
                prefix.push(first);
                prefix.extend_from_slice(&suffix);
                assert!(trie.is_empty());
                *trie = child;
            }
            (item, trie.is_empty())
        };
        if is_empty {
            self.c_remove(*first);
        }
        Ok(item)
    }

    async fn yield_all(
        &self,
        context: &mut Vec<u8>,
        co: &Co<(Vec<u8>, T), object_rainbow::Error>,
    ) -> object_rainbow::Result<()> {
        if let Some(value) = self.value.clone() {
            co.yield_((context.clone(), value)).await;
        }
        let len = context.len();
        for (first, point) in self.c_range(u8::MIN, u8::MAX) {
            {
                context.push(*first);
                let (prefix, trie) = point.fetch().await?;
                context.extend_from_slice(&prefix);
                Box::pin(trie.yield_all(context, co)).await?;
            }
            context.truncate(len);
        }
        Ok(())
    }

    async fn prefix_yield(
        &self,
        context: &mut Vec<u8>,
        key: &[u8],
        co: &Co<(Vec<u8>, T), object_rainbow::Error>,
    ) -> object_rainbow::Result<()> {
        let Some((first, key)) = key.split_first() else {
            self.yield_all(context, co).await?;
            return Ok(());
        };
        let Some(point) = self.c_get(*first) else {
            return Ok(());
        };
        let len = context.len();
        'done: {
            context.push(*first);
            let (prefix, trie) = point.fetch().await?;
            context.extend_from_slice(&prefix);
            if prefix.starts_with(key) {
                trie.yield_all(context, co).await?;
                break 'done;
            }
            let Some(key) = key.strip_prefix(prefix.as_slice()) else {
                break 'done;
            };
            Box::pin(trie.prefix_yield(context, key, co)).await?;
        }
        context.truncate(len);
        Ok(())
    }

    async fn range_yield(
        &self,
        context: &mut Vec<u8>,
        range_start: Bound<&[u8]>,
        range_end: Bound<&[u8]>,
        co: &Co<(Vec<u8>, T), object_rainbow::Error>,
    ) -> object_rainbow::Result<()> {
        if (range_start, range_end).contains(b"".as_slice())
            && let Some(value) = self.value.clone()
        {
            co.yield_((context.clone(), value)).await;
        }
        let min = match range_start {
            Bound::Included(x) | Bound::Excluded(x) => x.first().copied().unwrap_or(0),
            Bound::Unbounded => 0,
        };
        let max = match range_end {
            Bound::Included(x) => x.first().copied().unwrap_or(0),
            Bound::Excluded(x) => {
                if let Some(min) = x.first().copied() {
                    if x.len() == 1 {
                        if let Some(min) = min.checked_sub(1) {
                            min
                        } else {
                            return Ok(());
                        }
                    } else {
                        min
                    }
                } else {
                    return Ok(());
                }
            }
            Bound::Unbounded => 255,
        };
        let len = context.len();
        for (first, point) in self.c_range(min, max) {
            'done: {
                context.push(*first);
                let (prefix, trie) = point.fetch().await?;
                context.extend_from_slice(&prefix);
                let extra = &context[context.len() - prefix.len() - 1..];
                let start_bound = match range_start {
                    Bound::Included(x) => {
                        if x <= extra {
                            Bound::Unbounded
                        } else if let Some(suffix) = x.strip_prefix(extra) {
                            Bound::Included(suffix)
                        } else {
                            break 'done;
                        }
                    }
                    Bound::Excluded(x) => {
                        if x < extra {
                            Bound::Unbounded
                        } else if let Some(suffix) = x.strip_prefix(extra) {
                            Bound::Excluded(suffix)
                        } else {
                            break 'done;
                        }
                    }
                    Bound::Unbounded => Bound::Unbounded,
                };
                let end_bound = match range_end {
                    Bound::Included(x) => {
                        if x < extra {
                            break 'done;
                        } else if let Some(suffix) = x.strip_prefix(extra) {
                            Bound::Included(suffix)
                        } else {
                            Bound::Unbounded
                        }
                    }
                    Bound::Excluded(x) => {
                        if x <= extra {
                            break 'done;
                        } else if let Some(suffix) = x.strip_prefix(extra) {
                            Bound::Excluded(suffix)
                        } else {
                            Bound::Unbounded
                        }
                    }
                    Bound::Unbounded => Bound::Unbounded,
                };
                Box::pin(trie.range_yield(context, start_bound, end_bound, co)).await?;
            }
            context.truncate(len);
        }
        Ok(())
    }

    pub fn prefix_stream(
        &self,
        prefix: &[u8],
    ) -> impl Send + Stream<Item = object_rainbow::Result<(Vec<u8>, T)>> {
        try_stream(async |co| self.prefix_yield(&mut Vec::new(), prefix, &co).await)
    }

    pub fn range_stream<'a>(
        &'a self,
        range: impl 'a + Send + Sync + RangeBounds<&'a [u8]>,
    ) -> impl Send + Stream<Item = object_rainbow::Result<(Vec<u8>, T)>> {
        try_stream(async move |co| {
            self.range_yield(
                &mut Vec::new(),
                range.start_bound().cloned(),
                range.end_bound().cloned(),
                &co,
            )
            .await
        })
    }

    pub async fn count(&self) -> object_rainbow::Result<u64> {
        self.range_stream(..)
            .try_fold(0u64, async |ctr, _| Ok(ctr.saturating_add(1)))
            .await
    }

    pub async fn from_stream<K: AsRef<[u8]>>(
        stream: impl TryStream<Ok = (K, T), Error = object_rainbow::Error>,
    ) -> object_rainbow::Result<Self> {
        let mut trie = Self::default();
        let mut stream = pin!(stream.into_stream());
        while let Some((key, value)) = stream.try_next().await? {
            trie.insert(key.as_ref(), value).await?;
        }
        Ok(trie)
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
pub struct TrieMap<K, V> {
    key: ObjectMarker<K>,
    trie: Trie<V>,
}

impl<K, V: Clone> Clone for TrieMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            trie: self.trie.clone(),
        }
    }
}

impl<K, V> Default for TrieMap<K, V> {
    fn default() -> Self {
        Self {
            key: Default::default(),
            trie: Default::default(),
        }
    }
}

impl<K: ReflessObject + AsRef<[u8]>, V: 'static + Send + Sync + Clone> TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    pub async fn get(&self, key: &K) -> object_rainbow::Result<Option<V>> {
        self.trie.get(key.as_ref()).await
    }

    pub async fn insert(&mut self, key: &K, value: V) -> object_rainbow::Result<Option<V>> {
        self.trie.insert(key.as_ref(), value).await
    }

    pub fn is_empty(&self) -> bool {
        self.trie.is_empty()
    }

    pub async fn remove(&mut self, key: &K) -> object_rainbow::Result<Option<V>> {
        self.trie.remove(key.as_ref()).await
    }

    pub fn prefix_stream(
        &self,
        prefix: &[u8],
    ) -> impl Send + Stream<Item = object_rainbow::Result<(K, V)>> {
        self.trie
            .prefix_stream(prefix)
            .and_then(async |(key, value)| Ok((K::parse_slice_refless(&key)?, value)))
    }

    pub fn range_stream<'a>(
        &'a self,
        range: impl 'a + Send + Sync + RangeBounds<&'a K>,
    ) -> impl Send + Stream<Item = object_rainbow::Result<(K, V)>> {
        self.trie
            .range_stream((
                range.start_bound().cloned().map(|b| b.as_ref()),
                range.end_bound().cloned().map(|b| b.as_ref()),
            ))
            .and_then(async |(key, value)| Ok((K::parse_slice_refless(&key)?, value)))
    }

    pub async fn from_stream(
        stream: impl TryStream<Ok = (K, V), Error = object_rainbow::Error>,
    ) -> object_rainbow::Result<Self> {
        Ok(Self {
            key: Default::default(),
            trie: Trie::from_stream(stream).await?,
        })
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use smol::stream::StreamExt;
    use smol_macros::test;

    use crate::Trie;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut trie = Trie::<u8>::default();
        trie.insert(b"abc", 1).await?;
        assert_eq!(trie.get(b"abc").await?.unwrap(), 1);
        trie.insert(b"abd", 2).await?;
        assert_eq!(trie.get(b"abd").await?.unwrap(), 2);
        trie.insert(b"ab", 3).await?;
        assert_eq!(trie.get(b"ab").await?.unwrap(), 3);
        trie.insert(b"a", 4).await?;
        assert_eq!(trie.get(b"a").await?.unwrap(), 4);
        trie.insert(b"abce", 5).await?;
        assert_eq!(trie.get(b"abce").await?.unwrap(), 5);
        assert_eq!(
            trie.prefix_stream(b"")
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [
                (b"a".into(), 4),
                (b"ab".into(), 3),
                (b"abc".into(), 1),
                (b"abce".into(), 5),
                (b"abd".into(), 2),
            ],
        );
        assert_eq!(
            trie.prefix_stream(b"a")
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [
                (b"a".into(), 4),
                (b"ab".into(), 3),
                (b"abc".into(), 1),
                (b"abce".into(), 5),
                (b"abd".into(), 2),
            ],
        );
        assert_eq!(
            trie.prefix_stream(b"ab")
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [
                (b"ab".into(), 3),
                (b"abc".into(), 1),
                (b"abce".into(), 5),
                (b"abd".into(), 2),
            ],
        );
        assert_eq!(
            trie.range_stream(..).try_collect::<_, _, Vec<_>>().await?,
            [
                (b"a".into(), 4),
                (b"ab".into(), 3),
                (b"abc".into(), 1),
                (b"abce".into(), 5),
                (b"abd".into(), 2),
            ],
        );
        assert_eq!(
            trie.range_stream(..=b"abc".as_slice())
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [(b"a".into(), 4), (b"ab".into(), 3), (b"abc".into(), 1)],
        );
        assert_eq!(
            trie.range_stream(..b"abc".as_slice())
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [(b"a".into(), 4), (b"ab".into(), 3)],
        );
        assert_eq!(
            trie.range_stream(b"ab".as_slice()..)
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [
                (b"ab".into(), 3),
                (b"abc".into(), 1),
                (b"abce".into(), 5),
                (b"abd".into(), 2),
            ],
        );
        assert_eq!(
            trie.range_stream(b"ab".as_slice()..=b"abce".as_slice())
                .try_collect::<_, _, Vec<_>>()
                .await?,
            [(b"ab".into(), 3), (b"abc".into(), 1), (b"abce".into(), 5)],
        );
        assert_eq!(trie.remove(b"a").await?.unwrap(), 4);
        assert_eq!(trie.remove(b"ab").await?.unwrap(), 3);
        assert_eq!(trie.remove(b"abc").await?.unwrap(), 1);
        assert_eq!(trie.remove(b"abce").await?.unwrap(), 5);
        assert_eq!(trie.remove(b"abd").await?.unwrap(), 2);
        assert!(trie.is_empty());
        Ok(())
    }
}
