use std::task::{Context, Poll};

use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
use futures_util::{StreamExt, task::noop_waker_ref};

use super::*;

type Map<K, V> = BTreeMap<ByBytes<K>, V>;

impl<K: ReflessObject + AsRef<[u8]>, V: 'static + Send + Sync + Clone> TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    fn to_map(&self) -> object_rainbow::Result<Map<K, V>> {
        let future = self
            .range_stream(..)
            .map_ok(|(k, v)| (ByBytes(k), v))
            .try_collect();
        let future = pin!(future);
        match future.poll(&mut Context::from_waker(noop_waker_ref())) {
            Poll::Ready(map) => map,
            Poll::Pending => Err(object_rainbow::error_fetch!("not local")),
        }
    }

    fn from_map(map: Map<K, V>) -> object_rainbow::Result<Self> {
        let future =
            Self::from_stream(futures_util::stream::iter(map).map(|(ByBytes(k), v)| Ok((k, v))));
        let future = pin!(future);
        match future.poll(&mut Context::from_waker(noop_waker_ref())) {
            Poll::Ready(trie) => trie,
            Poll::Pending => Err(object_rainbow::error_fetch!("not local")),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ByBytes<K>(K);

impl<K: AsRef<[u8]>> PartialEq for ByBytes<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl<K: AsRef<[u8]>> Eq for ByBytes<K> {}

impl<K: AsRef<[u8]>> PartialOrd for ByBytes<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: AsRef<[u8]>> Ord for ByBytes<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

impl<K: ReflessObject + AsRef<[u8]> + Serialize, V: 'static + Send + Sync + Clone + Serialize>
    Serialize for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        self.to_map()
            .map_err(<S::Error as ::serde::ser::Error>::custom)?
            .serialize(serializer)
    }
}

impl<
    'de,
    K: ReflessObject + AsRef<[u8]> + DeserializeOwned,
    V: 'static + Send + Sync + Clone + DeserializeOwned,
> Deserialize<'de> for TrieMap<K, V>
where
    Option<V>: Traversible + InlineOutput,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::from_map(Map::deserialize(deserializer)?)
            .map_err(<D::Error as ::serde::de::Error>::custom)
    }
}
