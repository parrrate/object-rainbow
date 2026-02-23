use std::pin::Pin;

use object_rainbow::{
    Enum, Fetch, Hash, Inline, InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseInline,
    PointInput, PointVisitor, Size, Tagged, Tags, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_array_map::ArrayMap;
use object_rainbow_point::{IntoPoint, Point};

type OptionFuture<'a, T> =
    Pin<Box<dyn 'a + Send + Future<Output = object_rainbow::Result<Option<T>>>>>;

trait Amt<K> {
    type V: Send + Sync;
    fn insert(&mut self, key: K, value: Self::V) -> OptionFuture<'_, Self::V>;
    fn from_pair(a: (K, Self::V), b: (K, Self::V)) -> Self;
    fn get(&self, key: K) -> OptionFuture<'_, Self::V>;
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
/// what are you even doing at this point
struct DeepestLeaf<V = ()>(ArrayMap<V>);

impl<V: Send + Sync + Clone> Amt<u8> for DeepestLeaf<V> {
    type V = V;

    fn insert(&mut self, key: u8, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move { Ok(self.0.insert(key, value)) })
    }

    fn from_pair(a: (u8, Self::V), b: (u8, Self::V)) -> Self {
        Self([a, b].into())
    }

    fn get(&self, key: u8) -> OptionFuture<'_, Self::V> {
        Box::pin(async move { Ok(self.0.get(key).cloned()) })
    }
}

#[derive(Enum, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline)]
enum SubTree<T, K, V = <T as Amt<K>>::V> {
    Leaf(K, V),
    SubTree(Point<T>),
}

impl<T, K: Clone, V: Clone> Clone for SubTree<T, K, V> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(key, value) => Self::Leaf(key.clone(), value.clone()),
            Self::SubTree(point) => Self::SubTree(point.clone()),
        }
    }
}

impl<T: Amt<K, V: Clone> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Amt<K>
    for SubTree<T, K>
{
    type V = T::V;

    fn insert(&mut self, key: K, value: Self::V) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(xkey, xvalue) => Ok(if *xkey != key {
                    *self = Self::from_pair((xkey.clone(), xvalue.clone()), (key, value));
                    None
                } else {
                    Some(std::mem::replace(xvalue, value))
                }),
                SubTree::SubTree(sub) => sub.fetch_mut().await?.insert(key, value).await,
            }
        })
    }

    fn from_pair(a: (K, Self::V), b: (K, Self::V)) -> Self {
        Self::SubTree(T::from_pair(a, b).point())
    }

    fn get(&self, key: K) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(existing, value) => Ok((*existing == key).then(|| value.clone())),
                SubTree::SubTree(sub) => sub.fetch().await?.get(key).await,
            }
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

    fn get(&self, (key, rest): (u8, K)) -> OptionFuture<'_, Self::V> {
        Box::pin(async move {
            if let Some(sub) = self.0.get(key) {
                sub.get(rest).await
            } else {
                Ok(None)
            }
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
                fn to_output(&self, output: &mut dyn Output) {
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

                fn insert(&mut self, key: $k, value: Self::V) -> OptionFuture<'_, Self::V> {
                    self.0.insert(key, value)
                }

                fn from_pair(a: ($k, Self::V), b: ($k, Self::V)) -> Self {
                    Self(Amt::from_pair(a, b))
                }

                fn get(&self, key: $k) -> OptionFuture<'_, Self::V> {
                    self.0.get(key)
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
            .insert(hash_key(hash), value)
            .await
    }

    pub async fn get(&self, hash: Hash) -> object_rainbow::Result<Option<V>> {
        self.0.fetch().await?.get(hash_key(hash)).await
    }
}

fn hash_key(hash: Hash) -> K32 {
    let [
        x0,
        x1,
        x2,
        x3,
        x4,
        x5,
        x6,
        x7,
        x8,
        x9,
        x10,
        x11,
        x12,
        x13,
        x14,
        x15,
        x16,
        x17,
        x18,
        x19,
        x20,
        x21,
        x22,
        x23,
        x24,
        x25,
        x26,
        x27,
        x28,
        x29,
        x30,
        x31,
    ] = hash.into_bytes();
    (x0,(x1,(x2,(x3,(x4,(x5,(x6,(x7,(x8,(x9,(x10,(x11,(x12,(x13,(x14,(x15,(x16,(x17,(x18,(x19,(x20,(x21,(x22,(x23,(x24,(x25,(x26,(x27,(x28,(x29,(x30,(x31))))))))))))))))))))))))))))))))
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

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.get(hash).await?.is_some())
    }
}
