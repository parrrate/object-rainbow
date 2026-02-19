use std::collections::{BTreeMap, BTreeSet};

use futures_util::TryStreamExt;
use object_rainbow::{
    Fetch, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, Traversible,
};
use object_rainbow_chain_tree::ChainTree;
use object_rainbow_point::{IntoPoint, Point};

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
pub struct History<T, D>(ChainTree<(T, D)>);

impl<T, D> History<T, D> {
    pub const ROOT: Self = Self(ChainTree::EMPTY);
}

impl<T, D> Default for History<T, D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, D> History<T, D> {
    pub const fn new() -> Self {
        Self::ROOT
    }
}

pub trait Diff<Tree: Send>: Send {
    fn forward(self, tree: Tree) -> impl Send + Future<Output = object_rainbow::Result<Tree>>;
}

impl<T: Clone + Traversible + InlineOutput + Default, D: Clone + Traversible + Diff<T>>
    History<T, D>
{
    pub async fn commit(&mut self, diff: D) -> object_rainbow::Result<()> {
        let tree = self
            .0
            .prev()
            .await?
            .last()
            .await?
            .map(|(tree, _)| tree)
            .unwrap_or_default();
        let hash = tree.full_hash();
        let tree = diff.clone().forward(tree).await?;
        if hash != tree.full_hash() {
            self.0.push((tree, diff)).await?;
        }
        Ok(())
    }

    pub async fn forward(&self, other: &Self) -> object_rainbow::Result<()>
    where
        T: PartialEq,
    {
        other
            .0
            .diff(&self.0)
            .and_then(async |node| {
                let diff = node.value().1.clone();
                let tree = node
                    .prev()
                    .last()
                    .await?
                    .map(|(tree, _)| tree)
                    .unwrap_or_default();
                let tree = diff.clone().forward(tree).await?;
                if tree == node.value().0 {
                    Ok(())
                } else {
                    Err(object_rainbow::error_fetch!(
                        "diff doesn't match the new tree",
                    ))
                }
            })
            .try_collect()
            .await
    }
}

impl<D: Diff<T>, T: Send> Diff<T> for Vec<D> {
    fn forward(self, mut tree: T) -> impl Send + Future<Output = object_rainbow::Result<T>> {
        async move {
            for diff in self {
                tree = diff.forward(tree).await?;
            }
            Ok(tree)
        }
    }
}

impl<D: Diff<T> + Traversible, T: Send> Diff<T> for Point<D> {
    fn forward(self, tree: T) -> impl Send + Future<Output = object_rainbow::Result<T>> {
        async move { self.fetch().await?.forward(tree).await }
    }
}

impl<
    K: Send + Clone + Traversible + InlineOutput + Ord,
    V: Send + Clone + Traversible + InlineOutput,
> Diff<Point<BTreeMap<K, V>>> for (K, V)
{
    fn forward(
        self,
        tree: Point<BTreeMap<K, V>>,
    ) -> impl Send + Future<Output = object_rainbow::Result<Point<BTreeMap<K, V>>>> {
        async move {
            let mut tree = tree.fetch().await?;
            tree.insert(self.0, self.1);
            Ok(tree.point())
        }
    }
}

impl<T: Send + Clone + Traversible + InlineOutput + Ord> Diff<Point<BTreeSet<T>>> for (T,) {
    fn forward(
        self,
        tree: Point<BTreeSet<T>>,
    ) -> impl Send + Future<Output = object_rainbow::Result<Point<BTreeSet<T>>>> {
        async move {
            let mut tree = tree.fetch().await?;
            tree.insert(self.0);
            Ok(tree.point())
        }
    }
}
