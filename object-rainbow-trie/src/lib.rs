use std::collections::BTreeMap;

use object_rainbow::{
    Fetch, Inline, Object, Parse, Point, SimpleObject, Tagged, ToOutput, Topological,
    length_prefixed::LpBytes,
};

#[derive(ToOutput, Tagged, Topological, Parse, Clone)]
pub struct Trie<T, Extra = ()> {
    value: Option<T>,
    #[tags(skip)]
    children: BTreeMap<u8, Point<(LpBytes, Self), Extra>>,
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

impl<T: Clone> Trie<T>
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
        let Some(key) = key.strip_prefix(prefix.as_slice()) else {
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
            return Ok(None);
        };
        Box::pin(trie.insert(key, value)).await
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
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
        Ok(())
    }
}
