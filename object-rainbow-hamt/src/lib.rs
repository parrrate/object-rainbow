use std::pin::Pin;

use futures_util::TryStreamExt;
use object_rainbow::{
    Enum, Fetch, Hash, Inline, InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseInline,
    PointInput, PointVisitor, Singular, Size, SizeExt, Tagged, Tags, ToOutput, Topological,
    Traversible, assert_impl,
};
use object_rainbow_array_map::ArrayMap;
use object_rainbow_point::{IntoPoint, Point};

type ActionFuture<'a, T = ()> =
    Pin<Box<dyn 'a + Send + Future<Output = object_rainbow::Result<T>>>>;
type OptionFuture<'a, T> = ActionFuture<'a, Option<T>>;

trait Amt<K>: Sized {
    type V: Send + Sync;
    fn is_empty(&self) -> bool;
    fn insert(&mut self, key: K, value: Self::V) -> OptionFuture<'_, Self::V>;
    fn remove(&mut self, key: K) -> OptionFuture<'_, Self::V>;
    fn extract_only(&mut self) -> OptionFuture<'_, (K, Self::V)>;
    fn from_pair(a: (K, Self::V), b: (K, Self::V)) -> Self;
    fn get<'a, O: Send>(
        &'a self,
        key: K,
        f: impl 'a + Send + FnOnce(&Self::V) -> O,
    ) -> OptionFuture<'a, O>;
    fn append<'a>(&'a mut self, other: &'a mut Self) -> ActionFuture<'a>;
    fn intersect<'a>(&'a mut self, other: &'a Self) -> ActionFuture<'a>
    where
        Self::V: PartialEq;
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
/// what are you even doing at this point
struct DeepestLeaf<V = ()>(ArrayMap<V>);

impl<V: Send + Sync + Clone> Amt<u8> for DeepestLeaf<V> {
    type V = V;

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn insert(&mut self, key: u8, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move { Ok(self.0.insert(key, value)) })
    }

    fn remove(&mut self, key: u8) -> OptionFuture<'_, Self::V> {
        Box::pin(async move { Ok(self.0.remove(key)) })
    }

    fn extract_only(&mut self) -> OptionFuture<'_, (u8, Self::V)> {
        Box::pin(async move {
            Ok(if self.0.len() == 1 {
                self.0.pop_first()
            } else {
                None
            })
        })
    }

    fn from_pair(a: (u8, Self::V), b: (u8, Self::V)) -> Self {
        Self([a, b].into())
    }

    fn get<'a, O: Send>(
        &'a self,
        key: u8,
        f: impl 'a + Send + FnOnce(&Self::V) -> O,
    ) -> OptionFuture<'a, O> {
        Box::pin(async move { Ok(self.0.get(key).map(f)) })
    }

    fn append<'a>(&'a mut self, other: &'a mut Self) -> ActionFuture<'a> {
        Box::pin(async move {
            self.0.append(&mut other.0);
            Ok(())
        })
    }

    fn intersect<'a>(&'a mut self, other: &'a Self) -> ActionFuture<'a>
    where
        Self::V: PartialEq,
    {
        Box::pin(async move {
            self.0
                .retain(|key, value| other.0.get(key).is_some_and(|other| other == value));
            Ok(())
        })
    }
}

#[derive(
    Enum, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Default,
)]
enum SubTree<T, K, V = <T as Amt<K>>::V> {
    Leaf(K, V),
    Sub(Point<T>),
    #[default]
    Empty,
}

impl<T, K: Clone, V: Clone> Clone for SubTree<T, K, V> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(key, value) => Self::Leaf(key.clone(), value.clone()),
            Self::Sub(point) => Self::Sub(point.clone()),
            Self::Empty => Self::Empty,
        }
    }
}

impl<T: Amt<K, V: Clone> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Amt<K>
    for SubTree<T, K>
{
    type V = T::V;

    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn insert(&mut self, key: K, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            match self {
                Self::Leaf(xkey, xvalue) => Ok(if *xkey != key {
                    *self = Self::from_pair((xkey.clone(), xvalue.clone()), (key, value));
                    None
                } else {
                    Some(std::mem::replace(xvalue, value))
                }),
                Self::Sub(sub) => sub.fetch_mut().await?.insert(key, value).await,
                Self::Empty => Err(object_rainbow::error_consistency!(
                    "empty subtree? (invalid state)",
                )),
            }
        })
    }

    fn remove(&mut self, key: K) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            match self {
                Self::Leaf(xkey, _) => Ok(if *xkey != key {
                    None
                } else {
                    match std::mem::take(self) {
                        Self::Leaf(_, v) => Some(v),
                        _ => unreachable!(),
                    }
                }),
                Self::Sub(point) => {
                    let (value, extracted) = {
                        let sub = &mut *point.fetch_mut().await?;
                        let value = sub.remove(key).await?;
                        (value, sub.extract_only().await?)
                    };
                    if let Some((k, v)) = extracted {
                        *self = Self::Leaf(k, v);
                    }
                    Ok(value)
                }
                Self::Empty => Err(object_rainbow::error_consistency!(
                    "empty subtree? (invalid state)",
                )),
            }
        })
    }

    fn extract_only(&mut self) -> OptionFuture<'_, (K, Self::V)> {
        Box::pin(async move {
            match self {
                Self::Leaf(_, _) => match std::mem::take(self) {
                    Self::Leaf(k, v) => Ok(Some((k, v))),
                    _ => unreachable!(),
                },
                Self::Sub(point) => {
                    let extracted = point.fetch_mut().await?.extract_only().await?;
                    if let Some((k, v)) = extracted {
                        *self = Self::Empty;
                        Ok(Some((k, v)))
                    } else {
                        Ok(None)
                    }
                }
                Self::Empty => Err(object_rainbow::error_consistency!(
                    "empty subtree? (invalid state)",
                )),
            }
        })
    }

    fn from_pair(a: (K, Self::V), b: (K, Self::V)) -> Self {
        Self::Sub(T::from_pair(a, b).point())
    }

    fn get<'a, O: Send>(
        &'a self,
        key: K,
        f: impl 'a + Send + FnOnce(&Self::V) -> O,
    ) -> OptionFuture<'a, O> {
        Box::pin(async move {
            match self {
                Self::Leaf(existing, value) => Ok((*existing == key).then(|| f(value))),
                Self::Sub(sub) => sub.fetch().await?.get(key, f).await,
                Self::Empty => Err(object_rainbow::error_consistency!(
                    "empty subtree? (invalid state)",
                )),
            }
        })
    }

    fn append<'a>(&'a mut self, other: &'a mut Self) -> ActionFuture<'a> {
        Box::pin(async move {
            if matches!((&*self, &*other), (Self::Leaf(_, _), Self::Sub(_))) {
                std::mem::swap(self, other);
            }
            match (&mut *self, &mut *other) {
                (_, Self::Empty) => {}
                (Self::Empty, _) => std::mem::swap(self, other),
                (Self::Leaf(kl, vl), Self::Leaf(kr, vr)) if kl == kr => {
                    std::mem::swap(vl, vr);
                    *other = Self::Empty;
                }
                (Self::Leaf(_, _), Self::Leaf(_, _)) => {
                    match (std::mem::take(self), std::mem::take(other)) {
                        (Self::Leaf(kl, vl), Self::Leaf(kr, vr)) => {
                            *self = Self::Sub(T::from_pair((kl, vl), (kr, vr)).point());
                        }
                        _ => unreachable!(),
                    }
                }
                (Self::Leaf(_, _), Self::Sub(_)) => unreachable!(),
                (Self::Sub(point), Self::Leaf(_, _)) => match std::mem::take(other) {
                    Self::Leaf(key, value) => {
                        point.fetch_mut().await?.insert(key, value).await?;
                    }
                    _ => unreachable!(),
                },
                (Self::Sub(l), Self::Sub(r)) => {
                    if l.hash() != r.hash() {
                        l.fetch_mut()
                            .await?
                            .append(&mut *r.fetch_mut().await?)
                            .await?;
                    }
                    *other = Self::Empty;
                }
            }
            Ok(())
        })
    }

    fn intersect<'a>(&'a mut self, other: &'a Self) -> ActionFuture<'a>
    where
        Self::V: PartialEq,
    {
        Box::pin(async move {
            match (&mut *self, other) {
                (Self::Empty, _) => {}
                (_, Self::Empty) => *self = Self::Empty,
                (Self::Leaf(kl, vl), Self::Leaf(kr, vr)) if kl == kr && vl == vr => {}
                (Self::Leaf(_, _), Self::Leaf(_, _)) => *self = Self::Empty,
                (Self::Leaf(key, value), Self::Sub(sub))
                    if sub
                        .fetch()
                        .await?
                        .get(key.clone(), |existing| value == existing)
                        .await?
                        .unwrap_or_default() => {}
                (Self::Leaf(_, _), Self::Sub(_)) => *self = Self::Empty,
                (Self::Sub(sub), Self::Leaf(key, value))
                    if sub
                        .fetch()
                        .await?
                        .get(key.clone(), |existing| value == existing)
                        .await?
                        .unwrap_or_default() =>
                {
                    *self = other.clone()
                }
                (Self::Sub(_), Self::Leaf(_, _)) => *self = Self::Empty,
                (Self::Sub(sub), Self::Sub(other)) => {
                    if sub.hash() == other.hash() {
                        return Ok(());
                    }
                    let (is_empty, extracted) = {
                        let sub = &mut *sub.fetch_mut().await?;
                        let other = &other.fetch().await?;
                        sub.intersect(other).await?;
                        (sub.is_empty(), sub.extract_only().await?)
                    };
                    if is_empty {
                        assert!(extracted.is_none());
                        *self = Self::Empty;
                    }
                    if let Some((k, v)) = extracted {
                        assert!(!is_empty);
                        *self = Self::Leaf(k, v);
                    }
                }
            }
            Ok(())
        })
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
struct SetNode<T, K, V = <T as Amt<K>>::V>(ArrayMap<SubTree<T, K, V>>);

impl<T, K: Clone, V: Clone> Clone for SetNode<T, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, K, V> Default for SetNode<T, K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Amt<K, V: Clone> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Amt<(u8, K)>
    for SetNode<T, K>
{
    type V = T::V;

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn insert(&mut self, (key, rest): (u8, K), value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            if let Some(sub) = self.0.get_mut(key) {
                sub.insert(rest, value).await
            } else {
                assert!(self.0.insert(key, SubTree::Leaf(rest, value)).is_none());
                Ok(None)
            }
        })
    }

    fn remove(&mut self, (key, rest): (u8, K)) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            if let Some(sub) = self.0.get_mut(key) {
                let value = sub.remove(rest).await?;
                if sub.is_empty() {
                    self.0.remove(key);
                }
                Ok(value)
            } else {
                Ok(None)
            }
        })
    }

    fn extract_only(&mut self) -> OptionFuture<'_, ((u8, K), Self::V)> {
        Box::pin(async move {
            Ok(if self.0.len() == 1 {
                let (key, sub) = self.0.iter_mut().next().expect("must be len 1");
                if let Some((rest, v)) = sub.extract_only().await? {
                    self.0.remove(key);
                    Some(((key, rest), v))
                } else {
                    None
                }
            } else {
                None
            })
        })
    }

    fn from_pair(
        ((a, rest_a), value_a): ((u8, K), Self::V),
        ((b, rest_b), value_b): ((u8, K), Self::V),
    ) -> Self {
        if a == b {
            Self([(a, SubTree::from_pair((rest_a, value_a), (rest_b, value_b)))].into())
        } else {
            Self(
                [
                    (a, SubTree::Leaf(rest_a, value_a)),
                    (b, SubTree::Leaf(rest_b, value_b)),
                ]
                .into(),
            )
        }
    }

    fn get<'a, O: Send>(
        &'a self,
        (key, rest): (u8, K),
        f: impl 'a + Send + FnOnce(&Self::V) -> O,
    ) -> OptionFuture<'a, O> {
        Box::pin(async move {
            if let Some(sub) = self.0.get(key) {
                sub.get(rest, f).await
            } else {
                Ok(None)
            }
        })
    }

    fn append<'a>(&'a mut self, other: &'a mut Self) -> ActionFuture<'a> {
        Box::pin(async move {
            {
                let mut futures = futures_util::stream::FuturesUnordered::new();
                for (key, sub) in self.0.iter_mut() {
                    if let Some(mut other) = other.0.remove(key) {
                        futures.push(async move {
                            sub.append(&mut other)
                                .await
                                .map(|_| assert!(other.is_empty()))
                        });
                    }
                }
                while futures.try_next().await?.is_some() {}
            }
            while let Some((key, sub)) = other.0.pop_first() {
                assert!(!self.0.contains(key));
                self.0.insert(key, sub);
            }
            assert!(other.0.is_empty());
            Ok(())
        })
    }

    fn intersect<'a>(&'a mut self, other: &'a Self) -> ActionFuture<'a>
    where
        Self::V: PartialEq,
    {
        Box::pin(async move {
            {
                let mut futures = futures_util::stream::FuturesUnordered::new();
                for (key, sub) in self.0.iter_mut() {
                    if let Some(other) = other.0.get(key) {
                        futures.push(sub.intersect(other));
                    } else {
                        *sub = SubTree::Empty;
                    }
                }
                while futures.try_next().await?.is_some() {}
            }
            self.0.retain(|_, sub| !sub.is_empty());
            Ok(())
        })
    }
}

type K1 = u8;
type K2 = (u8, K1);
type K3 = (u8, K2);
type K4 = (u8, K3);
type K5 = (u8, K4);
type K6 = (u8, K5);
type K7 = (u8, K6);
type K8 = (u8, K7);
type K9 = (u8, K8);
type K10 = (u8, K9);
type K11 = (u8, K10);
type K12 = (u8, K11);
type K13 = (u8, K12);
type K14 = (u8, K13);
type K15 = (u8, K14);
type K16 = (u8, K15);
type K17 = (u8, K16);
type K18 = (u8, K17);
type K19 = (u8, K18);
type K20 = (u8, K19);
type K21 = (u8, K20);
type K22 = (u8, K21);
type K23 = (u8, K22);
type K24 = (u8, K23);
type K25 = (u8, K24);
type K26 = (u8, K25);
type K27 = (u8, K26);
type K28 = (u8, K27);
type K29 = (u8, K28);
type K30 = (u8, K29);
type K31 = (u8, K30);
type K32 = (u8, K31);

mod private {
    use super::*;
    type N1<V = ()> = DeepestLeaf<V>;

    macro_rules! next_node {
        ($prev:ident, $next:ident, $pk:ident, $k:ident) => {
            #[derive(Clone)]
            pub struct $next<V = ()>(SetNode<$prev<V>, $pk, V>);

            impl<V> Default for $next<V> {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            impl<V: ParseInline<I> + Inline<I::Extra>, I: PointInput<Extra: Send + Sync>> Parse<I>
                for $next<V>
            {
                fn parse(input: I) -> object_rainbow::Result<Self> {
                    Ok(Self(input.parse()?))
                }
            }

            impl<V: Tagged> Tagged for $next<V> {
                const TAGS: Tags = V::TAGS;
            }

            impl<V: InlineOutput> ToOutput for $next<V> {
                fn to_output(&self, output: &mut impl Output) {
                    self.0.to_output(output);
                }
            }

            impl<V: ListHashes> ListHashes for $next<V> {
                fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
                    self.0.list_hashes(f)
                }
            }

            impl<V: Traversible + InlineOutput> Topological for $next<V> {
                fn traverse(&self, visitor: &mut impl PointVisitor) {
                    self.0.traverse(visitor)
                }
            }

            impl<V: Traversible + InlineOutput + Clone> Amt<$k> for $next<V> {
                type V = V;

                fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }

                fn insert(&mut self, key: $k, value: Self::V) -> OptionFuture<'_, Self::V> {
                    self.0.insert(key, value)
                }

                fn remove(&mut self, key: $k) -> OptionFuture<'_, Self::V> {
                    self.0.remove(key)
                }

                fn extract_only(&mut self) -> OptionFuture<'_, ($k, Self::V)> {
                    self.0.extract_only()
                }

                fn from_pair(a: ($k, Self::V), b: ($k, Self::V)) -> Self {
                    Self(Amt::from_pair(a, b))
                }

                fn get<'a, O: Send>(
                    &'a self,
                    key: $k,
                    f: impl 'a + Send + FnOnce(&Self::V) -> O,
                ) -> OptionFuture<'a, O> {
                    self.0.get(key, f)
                }

                fn append<'a>(&'a mut self, other: &'a mut Self) -> ActionFuture<'a> {
                    self.0.append(&mut other.0)
                }

                fn intersect<'a>(&'a mut self, other: &'a Self) -> ActionFuture<'a>
                where
                    Self::V: PartialEq,
                {
                    self.0.intersect(&other.0)
                }
            }
        };
    }

    next_node!(N1, N2, K1, K2);
    next_node!(N2, N3, K2, K3);
    next_node!(N3, N4, K3, K4);
    next_node!(N4, N5, K4, K5);
    next_node!(N5, N6, K5, K6);
    next_node!(N6, N7, K6, K7);
    next_node!(N7, N8, K7, K8);
    next_node!(N8, N9, K8, K9);
    next_node!(N9, N10, K9, K10);
    next_node!(N10, N11, K10, K11);
    next_node!(N11, N12, K11, K12);
    next_node!(N12, N13, K12, K13);
    next_node!(N13, N14, K13, K14);
    next_node!(N14, N15, K14, K15);
    next_node!(N15, N16, K15, K16);
    next_node!(N16, N17, K16, K17);
    next_node!(N17, N18, K17, K18);
    next_node!(N18, N19, K18, K19);
    next_node!(N19, N20, K19, K20);
    next_node!(N20, N21, K20, K21);
    next_node!(N21, N22, K21, K22);
    next_node!(N22, N23, K22, K23);
    next_node!(N23, N24, K23, K24);
    next_node!(N24, N25, K24, K25);
    next_node!(N25, N26, K25, K26);
    next_node!(N26, N27, K26, K27);
    next_node!(N27, N28, K27, K28);
    next_node!(N28, N29, K28, K29);
    next_node!(N29, N30, K29, K30);
    next_node!(N30, N31, K30, K31);
    next_node!(N31, N32, K31, K32);
}

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
pub struct HamtMap<V>(Point<private::N32<V>>);

assert_impl!(
    impl<V, E> Inline<E> for HamtMap<V>
    where
        E: 'static + Send + Sync + Clone,
        V: Inline<E>,
    {
    }
);

impl<V> Clone for HamtMap<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V> PartialEq for HamtMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V> Eq for HamtMap<V> {}

impl<V: Traversible + InlineOutput + Clone> Default for HamtMap<V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<V: Traversible + InlineOutput + Clone> HamtMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash, value: V) -> object_rainbow::Result<Option<V>> {
        self.0
            .fetch_mut()
            .await?
            .insert(hash.reinterpret(), value)
            .await
    }

    pub async fn remove(&mut self, hash: Hash) -> object_rainbow::Result<Option<V>> {
        self.0.fetch_mut().await?.remove(hash.reinterpret()).await
    }

    pub async fn get(&self, hash: Hash) -> object_rainbow::Result<Option<V>> {
        self.0
            .fetch()
            .await?
            .get(hash.reinterpret(), |value| value.clone())
            .await
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0
            .fetch()
            .await?
            .get(hash.reinterpret(), |_| {})
            .await
            .map(|o| o.is_some())
    }

    pub async fn append(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        if self.0.hash() == other.0.hash() {
            other.clear();
            Ok(())
        } else {
            self.0
                .fetch_mut()
                .await?
                .append(&mut *other.0.fetch_mut().await?)
                .await
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_default()
    }

    pub fn clear(&mut self) {
        std::mem::take(self);
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
    Size,
    MaybeHasNiche,
    Clone,
    Default,
    PartialEq,
    Eq,
)]
pub struct HamtSet(HamtMap<()>);

assert_impl!(
    impl<E> Inline<E> for HamtSet where E: 'static + Send + Sync + Clone {}
);

impl HamtSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.insert(hash, ()).await?.is_none())
    }

    pub async fn remove(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.remove(hash).await?.is_some())
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0.contains(hash).await
    }

    pub async fn append(&mut self, other: &mut Self) -> object_rainbow::Result<()> {
        self.0.append(&mut other.0).await
    }

    pub async fn intersect(&mut self, other: &Self) -> object_rainbow::Result<()> {
        self.0
            .0
            .fetch_mut()
            .await?
            .intersect(&other.0.0.fetch().await?)
            .await
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        std::mem::take(self);
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::{FullHash, numeric::Le};
    use smol_macros::test;

    use crate::{HamtMap, HamtSet};

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut map = HamtMap::<Le<u16>>::new();
        let empty_hash = map.full_hash();
        for i in (0u16..=10_000).map(Le) {
            map.insert(i.full_hash(), i).await?;
        }
        for i in (0u16..=10_000).map(Le) {
            assert_eq!(map.get(i.full_hash()).await?, Some(i));
        }
        for i in (0u16..=10_000).map(Le) {
            map.remove(i.full_hash()).await?;
        }
        assert_eq!(map.full_hash(), empty_hash);
        Ok(())
    }

    #[apply(test!)]
    async fn intersect() -> object_rainbow::Result<()> {
        let mut l = HamtSet::new();
        for i in 0u8..=254 {
            l.insert((i, 1u8).full_hash()).await?;
            l.insert((i, 3u8).full_hash()).await?;
        }
        let mut r = HamtSet::new();
        for i in 0u8..=254 {
            r.insert((i, 2u8).full_hash()).await?;
            r.insert((i, 3u8).full_hash()).await?;
        }
        l.intersect(&r).await?;
        for i in 0u8..=254 {
            assert!(!l.contains((i, 1u8).full_hash()).await?);
            assert!(!l.contains((i, 2u8).full_hash()).await?);
            assert!(l.contains((i, 3u8).full_hash()).await?);
        }
        let mut r = HamtSet::new();
        for i in 0u8..=254 {
            r.insert((i, 1u8).full_hash()).await?;
            r.insert((i, 2u8).full_hash()).await?;
        }
        l.intersect(&r).await?;
        assert!(l.is_empty());
        Ok(())
    }
}
