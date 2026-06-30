#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

use futures_util::TryStreamExt;
use object_rainbow::{
    Component, Fetch, Inline, InlineOutput, ListHashes, MaybeHasNiche, Object, Parse, ParseInline,
    Size, Tagged, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_apply::Apply;
use object_rainbow_chain_tree::ChainTree;
use object_rainbow_point::Point;

pub mod enforce_unique;
pub mod remap;
pub mod skip;
#[cfg(feature = "unique-diffs")]
pub mod unique_diffs;

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
pub struct History<T, D>(ChainTree<(T, D)>);

assert_impl!(
    impl<T, D, E> Inline<E> for History<T, D>
    where
        E: 'static + Send + Sync + Clone,
        T: Inline<E>,
        D: Object<E>,
    {
    }
);

impl<T, D> Clone for History<T, D> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

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

impl<T: Component + Default + Apply<D>, D: Clone + Traversible> History<T, D> {
    pub async fn commit(&mut self, diff: D) -> object_rainbow::Result<T::Output> {
        let mut tree = self.tree().await?;
        let hash = tree.full_hash();
        let o = tree.apply(diff.clone()).await?;
        if hash != tree.full_hash() {
            self.0.push((tree, diff)).await?;
        }
        Ok(o)
    }

    pub async fn check_forward(&self, other: &Self) -> object_rainbow::Result<()> {
        other
            .0
            .diff(&self.0)
            .and_then(async |node| {
                let diff = node.value().1.clone();
                let mut tree = node
                    .prev()
                    .last()
                    .await?
                    .map(|(tree, _)| tree)
                    .unwrap_or_default();
                let hash = tree.full_hash();
                tree.apply(diff).await?;
                let new_hash = tree.full_hash();
                if new_hash == hash {
                    Err(object_rainbow::error_consistency!("noop diff"))
                } else if new_hash == node.value().0.full_hash() {
                    Ok(())
                } else {
                    Err(object_rainbow::error_consistency!(
                        "diff doesn't match the new tree",
                    ))
                }
            })
            .try_collect()
            .await
    }

    pub async fn forward(&mut self, other: Self) -> object_rainbow::Result<()> {
        self.check_forward(&other).await?;
        self.forward(other).await
    }

    pub async fn tree(&self) -> object_rainbow::Result<T> {
        Ok(self
            .0
            .last()
            .await?
            .map(|(tree, _)| tree)
            .unwrap_or_default())
    }

    pub async fn rebase(&mut self, base: &Self) -> object_rainbow::Result<()> {
        let mut base = base.clone();
        base.rebase_other(self).await?;
        *self = base;
        Ok(())
    }

    pub async fn rebase_other(&mut self, other: &Self) -> object_rainbow::Result<()> {
        let common_ancestor = self.0.common_ancestor(&[&other.0]).await?;
        let diff = other
            .0
            .diff_backwards(&common_ancestor)
            .map_ok(|node| node.value().1.clone())
            .try_collect::<Vec<_>>()
            .await?;
        for diff in diff.into_iter().rev() {
            self.commit(diff).await?;
        }
        Ok(())
    }
}

impl<T: Component + Default + Apply<D>, D: Clone + Traversible> Apply<D> for History<T, D> {
    type Output = T::Output;

    fn apply(
        &mut self,
        diff: D,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.commit(diff)
    }
}

#[derive(
    Debug,
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
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct FromIter<T>(pub T);

impl<T: Apply<D>, D: Send, I: Send + IntoIterator<Item = D, IntoIter: Send>> Apply<I>
    for FromIter<T>
{
    type Output = Vec<T::Output>;

    async fn apply(&mut self, diff: I) -> object_rainbow::Result<Self::Output> {
        let mut output = Vec::new();
        for diff in diff {
            output.push(self.0.apply(diff).await?);
        }
        Ok(output)
    }
}

#[derive(
    Debug,
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
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct Points<T>(pub T);

impl<T: Apply<D>, D: Send + Traversible> Apply<Point<D>> for Points<T> {
    type Output = T::Output;

    async fn apply(&mut self, diff: Point<D>) -> object_rainbow::Result<Self::Output> {
        self.0.apply(diff.fetch().await?).await
    }
}
