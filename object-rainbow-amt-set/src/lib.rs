use std::pin::Pin;

use object_rainbow::{
    Enum, Fetch, Hash, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged,
    ToOutput, Topological, Traversible,
};
use object_rainbow_array_map::{ArrayMap, ArraySet};
use object_rainbow_point::{IntoPoint, Point};

type BoolFuture<'a> = Pin<Box<dyn 'a + Send + Future<Output = object_rainbow::Result<bool>>>>;

trait Tree<K> {
    fn insert(&mut self, key: K) -> BoolFuture<'_>;
    fn from_pair(a: K, b: K) -> Self;
    fn contains(&self, key: K) -> BoolFuture<'_>;
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
/// what are you even doing at this point
struct DeepestLeaf(ArraySet);

impl Tree<u8> for DeepestLeaf {
    fn insert(&mut self, key: u8) -> BoolFuture<'_> {
        Box::pin(async move { Ok(self.0.insert(key)) })
    }

    fn from_pair(a: u8, b: u8) -> Self {
        Self([a, b].into())
    }

    fn contains(&self, key: u8) -> BoolFuture<'_> {
        Box::pin(async move { Ok(self.0.contains(key)) })
    }
}

#[derive(Enum, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline)]
enum SubTree<T, K> {
    Leaf(K),
    SubTree(Point<T>),
}

impl<T, K: Clone> Clone for SubTree<T, K> {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(arg0) => Self::Leaf(arg0.clone()),
            Self::SubTree(arg0) => Self::SubTree(arg0.clone()),
        }
    }
}

impl<T: Tree<K> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Tree<K>
    for SubTree<T, K>
{
    fn insert(&mut self, key: K) -> BoolFuture<'_> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(existing) => Ok(if *existing != key {
                    *self = Self::from_pair(existing.clone(), key);
                    true
                } else {
                    false
                }),
                SubTree::SubTree(sub) => sub.fetch_mut().await?.insert(key).await,
            }
        })
    }

    fn from_pair(a: K, b: K) -> Self {
        Self::SubTree(T::from_pair(a, b).point())
    }

    fn contains(&self, key: K) -> BoolFuture<'_> {
        Box::pin(async move {
            match self {
                SubTree::Leaf(existing) => Ok(*existing == key),
                SubTree::SubTree(sub) => sub.fetch().await?.contains(key).await,
            }
        })
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
struct SetNode<T, K>(ArrayMap<SubTree<T, K>>);

impl<T, K: Clone> Clone for SetNode<T, K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, K> Default for SetNode<T, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Tree<K> + Clone + Traversible, K: Send + Sync + PartialEq + Clone> Tree<(u8, K)>
    for SetNode<T, K>
{
    fn insert(&mut self, (key, rest): (u8, K)) -> BoolFuture<'_> {
        Box::pin(async move {
            if let Some(sub) = self.0.get_mut(key) {
                sub.insert(rest).await
            } else {
                assert!(self.0.insert(key, SubTree::Leaf(rest)).is_none());
                Ok(true)
            }
        })
    }

    fn from_pair((a, rest_a): (u8, K), (b, rest_b): (u8, K)) -> Self {
        if a == b {
            Self([(a, SubTree::from_pair(rest_a, rest_b))].into())
        } else {
            Self([(a, SubTree::Leaf(rest_a)), (b, SubTree::Leaf(rest_b))].into())
        }
    }

    fn contains(&self, (key, rest): (u8, K)) -> BoolFuture<'_> {
        Box::pin(async move {
            if let Some(sub) = self.0.get(key) {
                sub.contains(rest).await
            } else {
                Ok(false)
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
    type N1 = DeepestLeaf;

    macro_rules! next_node {
        ($prev:ident, $next:ident, $pk:ident, $k:ident) => {
            #[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone, Default)]
            pub struct $next(SetNode<$prev, $pk>);

            impl Tree<$k> for $next {
                fn insert(&mut self, key: $k) -> BoolFuture<'_> {
                    self.0.insert(key)
                }

                fn from_pair(a: $k, b: $k) -> Self {
                    Self(Tree::from_pair(a, b))
                }

                fn contains(&self, key: $k) -> BoolFuture<'_> {
                    self.0.contains(key)
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
    ToOutput,
    InlineOutput,
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
pub struct AmtSet(Point<private::N32>);

impl Tagged for AmtSet {}

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

impl AmtSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0.fetch_mut().await?.insert(hash_key(hash)).await
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0.fetch().await?.contains(hash_key(hash)).await
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::ToOutput;
    use smol_macros::test;

    use crate::AmtSet;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut tree = AmtSet::default();
        assert!(tree.insert(1u8.data_hash()).await?);
        assert!(tree.contains(1u8.data_hash()).await?);
        assert!(!tree.insert(1u8.data_hash()).await?);
        assert!(tree.contains(1u8.data_hash()).await?);
        assert!(tree.insert(2u8.data_hash()).await?);
        assert!(tree.contains(1u8.data_hash()).await?);
        assert!(tree.contains(2u8.data_hash()).await?);
        assert!(!tree.insert(2u8.data_hash()).await?);
        assert!(tree.contains(1u8.data_hash()).await?);
        assert!(tree.contains(2u8.data_hash()).await?);
        Ok(())
    }
}
