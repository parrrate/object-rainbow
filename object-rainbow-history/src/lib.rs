use futures_util::TryStreamExt;
use object_rainbow::{
    Fetch, Inline, InlineOutput, ListHashes, MaybeHasNiche, Object, Parse, ParseInline, Size,
    Tagged, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_chain_tree::ChainTree;
use object_rainbow_point::Point;

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

pub trait Forward<Diff: Send>: Send {
    fn forward(&mut self, diff: Diff) -> impl Send + Future<Output = object_rainbow::Result<()>>;
}

impl<T: Clone + Traversible + InlineOutput + Default + Forward<D>, D: Clone + Traversible>
    History<T, D>
{
    pub async fn commit(&mut self, diff: D) -> object_rainbow::Result<()> {
        let mut tree = self.tree().await?;
        let hash = tree.full_hash();
        tree.forward(diff.clone()).await?;
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
                let mut tree = node
                    .prev()
                    .last()
                    .await?
                    .map(|(tree, _)| tree)
                    .unwrap_or_default();
                let hash = tree.full_hash();
                tree.forward(diff).await?;
                if hash == tree.full_hash() {
                    Err(object_rainbow::error_fetch!("noop diff"))
                } else if tree == node.value().0 {
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
)]
pub struct Compat<T>(pub T);

impl<T: Forward<D>, D: Send> Forward<Vec<D>> for Compat<T> {
    async fn forward(&mut self, diff: Vec<D>) -> object_rainbow::Result<()> {
        for diff in diff {
            self.0.forward(diff).await?;
        }
        Ok(())
    }
}

impl<T: Forward<D>, D: Send + Traversible> Forward<Point<D>> for Compat<T> {
    async fn forward(&mut self, diff: Point<D>) -> object_rainbow::Result<()> {
        self.0.forward(diff.fetch().await?).await
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
    Copy,
    PartialEq,
)]
pub struct DiscardHeader<T>(pub T);

impl<T: Forward<D>, D: Send, H: Send> Forward<(H, D)> for DiscardHeader<T> {
    fn forward(
        &mut self,
        (_, diff): (H, D),
    ) -> impl Send + Future<Output = object_rainbow::Result<()>> {
        self.0.forward(diff)
    }
}
