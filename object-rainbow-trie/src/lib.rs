use std::collections::BTreeMap;

use futures_core::Stream;
use genawaiter_try_stream::{Co, try_stream};
use object_rainbow::{
    Fetch, Inline, Object, Parse, Point, SimpleObject, Tagged, ToOutput, Topological,
    length_prefixed::LpBytes,
};

#[derive(ToOutput, Tagged, Topological, Parse, Clone)]
pub struct Trie<T> {
    value: Option<T>,
    #[tags(skip)]
    children: BTreeMap<u8, Point<(LpBytes, Self)>>,
}

impl<T> Object for Trie<T> where Option<T>: Inline {}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            children: Default::default(),
        }
    }
}

impl<T: 'static + Send + Sync + Clone> Trie<T>
where
    Option<T>: Inline,
{
    pub async fn get(&self, key: &[u8]) -> object_rainbow::Result<Option<T>> {
        let Some((first, key)) = key.split_first() else {
            return Ok(self.value.clone());
        };
        let Some(point) = self.children.get(first) else {
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
        let Some(point) = self.children.get_mut(first) else {
            self.children.insert(
                *first,
                (
                    LpBytes(key.into()),
                    Self {
                        value: Some(value),
                        children: Default::default(),
                    },
                )
                    .point(),
            );
            return Ok(None);
        };
        let (prefix, trie) = &mut *point.fetch_mut().await?;
        if let Some(key) = key.strip_prefix(prefix.as_slice()) {
            return Box::pin(trie.insert(key, value)).await;
        }
        if let Some(suffix) = prefix.strip_prefix(key) {
            let child = std::mem::replace(
                trie,
                Self {
                    value: Some(value),
                    children: Default::default(),
                },
            );
            let (first, suffix) = suffix.split_first().expect("must be at least 1");
            trie.children
                .insert(*first, (LpBytes(suffix.into()), child).point());
            prefix.0.truncate(key.len());
        } else {
            let common = prefix.iter().zip(key).take_while(|(a, b)| a == b).count();
            let child = std::mem::take(trie);
            trie.children.insert(
                prefix[common],
                (LpBytes(prefix[common + 1..].to_vec()), child).point(),
            );
            trie.children.insert(
                key[common],
                (
                    LpBytes(prefix[common + 1..].to_vec()),
                    Self {
                        value: Some(value),
                        children: Default::default(),
                    },
                )
                    .point(),
            );
            prefix.0.truncate(common);
        }
        Ok(None)
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty() && self.value.is_none()
    }

    pub async fn remove(&mut self, key: &[u8]) -> object_rainbow::Result<Option<T>> {
        let Some((first, key)) = key.split_first() else {
            return Ok(self.value.take());
        };
        let (item, is_empty) = {
            let Some(point) = self.children.get_mut(first) else {
                return Ok(None);
            };
            let (prefix, trie) = &mut *point.fetch_mut().await?;
            let Some(key) = key.strip_prefix(prefix.as_slice()) else {
                return Ok(None);
            };
            let item = Box::pin(trie.remove(key)).await?;
            if trie.value.is_none()
                && trie.children.len() < 2
                && let Some((first, point)) = trie.children.pop_first()
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
            self.children.remove(first);
        }
        Ok(item)
    }

    async fn prefix_yield(
        &self,
        context: &mut Vec<u8>,
        key: &[u8],
        co: &Co<(Vec<u8>, T), object_rainbow::Error>,
    ) -> object_rainbow::Result<()> {
        let len = context.len();
        let Some((first, key)) = key.split_first() else {
            if let Some(value) = self.value.clone() {
                co.yield_((context.clone(), value)).await;
            }
            for (first, point) in &self.children {
                {
                    context.push(*first);
                    let (prefix, trie) = point.fetch().await?;
                    context.extend_from_slice(&prefix);
                    Box::pin(trie.prefix_yield(context, b"", co)).await?;
                }
                context.truncate(len);
            }
            return Ok(());
        };
        let Some(point) = self.children.get(first) else {
            return Ok(());
        };
        'done: {
            context.push(*first);
            let (prefix, trie) = point.fetch().await?;
            context.extend_from_slice(&prefix);
            if prefix.starts_with(key) {
                Box::pin(trie.prefix_yield(context, b"", co)).await?;
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

    pub fn prefix_stream(
        &self,
        key: &[u8],
    ) -> impl Stream<Item = object_rainbow::Result<(Vec<u8>, T)>> {
        try_stream(async |co| self.prefix_yield(&mut Vec::new(), key, &co).await)
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
        assert_eq!(trie.remove(b"a").await?.unwrap(), 4);
        assert_eq!(trie.remove(b"ab").await?.unwrap(), 3);
        assert_eq!(trie.remove(b"abc").await?.unwrap(), 1);
        assert_eq!(trie.remove(b"abce").await?.unwrap(), 5);
        assert_eq!(trie.remove(b"abd").await?.unwrap(), 2);
        assert!(trie.is_empty());
        Ok(())
    }
}
