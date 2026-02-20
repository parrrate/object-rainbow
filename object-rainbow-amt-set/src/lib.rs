#![recursion_limit = "512"]

use std::ops::{Add, Sub};

use generic_array::{ArrayLength, GenericArray, sequence::Split};
use object_rainbow::{
    Enum, Fetch, Hash, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged,
    ToOutput, Topological, Traversible,
};
use object_rainbow_array_map::{ArrayMap, ArraySet};
use object_rainbow_point::{IntoPoint, Point};
use typenum::{
    Add1, B1, U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16, U17, U18, U19,
    U20, U21, U22, U23, U24, U25, U26, U27, U28, U29, U30, U31, U32,
};

trait Tree {
    type N: Send + Sync + ArrayLength;
    fn insert(
        &mut self,
        key: GenericArray<u8, Self::N>,
    ) -> impl Future<Output = object_rainbow::Result<bool>>;
    fn from_pair(a: GenericArray<u8, Self::N>, b: GenericArray<u8, Self::N>) -> Self;
    fn contains(
        &self,
        key: GenericArray<u8, Self::N>,
    ) -> impl Future<Output = object_rainbow::Result<bool>>;
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
struct DeepestLeaf(ArraySet);

impl Tree for DeepestLeaf {
    type N = U1;

    async fn insert(&mut self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        let [key] = <[u8; 1]>::from(key);
        Ok(self.0.insert(key))
    }

    fn from_pair(a: GenericArray<u8, Self::N>, b: GenericArray<u8, Self::N>) -> Self {
        let [a] = <[u8; 1]>::from(a);
        let [b] = <[u8; 1]>::from(b);
        Self([a, b].into())
    }

    async fn contains(&self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        let [key] = <[u8; 1]>::from(key);
        Ok(self.0.contains(key))
    }
}

#[derive(
    Enum, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Clone,
)]
enum SubTree<T: Tree> {
    Leaf(GenericArray<u8, T::N>),
    SubTree(Point<T>),
}

impl<T: Tree + Clone + Traversible> Tree for SubTree<T> {
    type N = T::N;

    async fn insert(&mut self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        match self {
            SubTree::Leaf(existing) => Ok(if *existing != key {
                *self = Self::from_pair(existing.clone(), key);
                true
            } else {
                false
            }),
            SubTree::SubTree(sub) => sub.fetch_mut().await?.insert(key).await,
        }
    }

    fn from_pair(a: GenericArray<u8, Self::N>, b: GenericArray<u8, Self::N>) -> Self {
        Self::SubTree(T::from_pair(a, b).point())
    }

    async fn contains(&self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        match self {
            SubTree::Leaf(existing) => Ok(*existing == key),
            SubTree::SubTree(sub) => sub.fetch().await?.contains(key).await,
        }
    }
}

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone)]
struct SetNode<T: Tree>(ArrayMap<SubTree<T>>);

impl<T: Tree> Default for SetNode<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<
    T: Tree<N: Add<B1, Output: Send + Sync + ArrayLength + Sub<U1, Output = T::N>>>
        + Clone
        + Traversible,
> Tree for SetNode<T>
{
    type N = Add1<T::N>;

    async fn insert(&mut self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        let (key, rest) = Split::<u8, U1>::split(key);
        let [key] = <[u8; 1]>::from(key);
        if let Some(sub) = self.0.get_mut(key) {
            sub.insert(rest).await
        } else {
            assert!(self.0.insert(key, SubTree::Leaf(rest)).is_none());
            Ok(true)
        }
    }

    fn from_pair(a: GenericArray<u8, Self::N>, b: GenericArray<u8, Self::N>) -> Self {
        let (a, rest_a) = Split::<u8, U1>::split(a);
        let (b, rest_b) = Split::<u8, U1>::split(b);
        let [a] = <[u8; 1]>::from(a);
        let [b] = <[u8; 1]>::from(b);
        if a == b {
            Self([(a, SubTree::from_pair(rest_a, rest_b))].into())
        } else {
            Self([(a, SubTree::Leaf(rest_a)), (b, SubTree::Leaf(rest_b))].into())
        }
    }

    async fn contains(&self, key: GenericArray<u8, Self::N>) -> object_rainbow::Result<bool> {
        let (key, rest) = Split::<u8, U1>::split(key);
        let [key] = <[u8; 1]>::from(key);
        if let Some(sub) = self.0.get(key) {
            sub.contains(rest).await
        } else {
            Ok(false)
        }
    }
}

mod private {
    use super::*;
    type N1 = DeepestLeaf;

    macro_rules! next_node {
        ($prev:ident, $next:ident, $size:ident) => {
            #[derive(ToOutput, Tagged, ListHashes, Topological, Parse, Clone, Default)]
            pub struct $next(SetNode<$prev>);

            impl Tree for $next {
                type N = $size;

                fn insert(
                    &mut self,
                    key: GenericArray<u8, Self::N>,
                ) -> impl Future<Output = object_rainbow::Result<bool>> {
                    self.0.insert(key)
                }

                fn from_pair(a: GenericArray<u8, Self::N>, b: GenericArray<u8, Self::N>) -> Self {
                    Self(Tree::from_pair(a, b))
                }

                fn contains(
                    &self,
                    key: GenericArray<u8, Self::N>,
                ) -> impl Future<Output = object_rainbow::Result<bool>> {
                    self.0.contains(key)
                }
            }
        };
    }

    next_node!(N1, N2, U2);
    next_node!(N2, N3, U3);
    next_node!(N3, N4, U4);
    next_node!(N4, N5, U5);
    next_node!(N5, N6, U6);
    next_node!(N6, N7, U7);
    next_node!(N7, N8, U8);
    next_node!(N8, N9, U9);
    next_node!(N9, N10, U10);
    next_node!(N10, N11, U11);
    next_node!(N11, N12, U12);
    next_node!(N12, N13, U13);
    next_node!(N13, N14, U14);
    next_node!(N14, N15, U15);
    next_node!(N15, N16, U16);
    next_node!(N16, N17, U17);
    next_node!(N17, N18, U18);
    next_node!(N18, N19, U19);
    next_node!(N19, N20, U20);
    next_node!(N20, N21, U21);
    next_node!(N21, N22, U22);
    next_node!(N22, N23, U23);
    next_node!(N23, N24, U24);
    next_node!(N24, N25, U25);
    next_node!(N25, N26, U26);
    next_node!(N26, N27, U27);
    next_node!(N27, N28, U28);
    next_node!(N28, N29, U29);
    next_node!(N29, N30, U30);
    next_node!(N30, N31, U31);
    next_node!(N31, N32, U32);
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
)]
pub struct AmtSet(Point<private::N32>);

impl AmtSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0
            .fetch_mut()
            .await?
            .insert(hash.into_bytes().into())
            .await
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.0
            .fetch()
            .await?
            .contains(hash.into_bytes().into())
            .await
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
