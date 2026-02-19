use futures_util::TryStreamExt;
use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, Traversible,
};
use object_rainbow_chain_tree::ChainTree;

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

pub trait Diff<Tree>: Send {
    fn forward(
        self,
        tree: Option<Tree>,
    ) -> impl Send + Future<Output = object_rainbow::Result<Tree>>;
}

impl<T: Clone, D: Clone + Diff<T>> History<T, D>
where
    (T, D): Traversible,
{
    pub async fn commit(&mut self, diff: D) -> object_rainbow::Result<()> {
        let tree = self.0.prev().await?.last().await?.map(|(tree, _)| tree);
        let tree = diff.clone().forward(tree).await?;
        self.0.push((tree, diff)).await?;
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
                let tree = node.prev().last().await?.map(|(tree, _)| tree);
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

#[derive(ToOutput, Tagged, ListHashes, Topological, Parse)]
pub struct Bulk<D>(Vec<D>);

impl<D: Diff<T>, T: Send> Diff<T> for Bulk<D> {
    fn forward(
        self,
        mut tree: Option<T>,
    ) -> impl Send + Future<Output = object_rainbow::Result<T>> {
        async move {
            for diff in self.0 {
                tree = Some(diff.forward(tree).await?);
            }
            tree.ok_or_else(|| object_rainbow::error_fetch!("empty diff"))
        }
    }
}
