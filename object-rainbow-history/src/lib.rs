use futures_util::TryStreamExt;
use object_rainbow::{
    Equivalent, Fetch, Inline, InlineOutput, ListHashes, MaybeHasNiche, Object, Parse, ParseInline,
    Size, Tagged, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_chain_tree::ChainTree;
use object_rainbow_point::Point;

#[cfg(feature = "trie")]
mod trie;

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
    type Output: Send;
    /// Must stay isomorphic under [`Equivalent`] conversions.
    fn forward(
        &mut self,
        diff: Diff,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>>;
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
                    Err(object_rainbow::error_consistency!("noop diff"))
                } else if tree == node.value().0 {
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

assert_impl!(
    impl<T, E> Inline<E> for Compat<T>
    where
        T: Inline<E>,
        E: 'static + Send + Sync + Clone,
    {
    }
);

impl<T: Forward<D>, D: Send> Forward<Vec<D>> for Compat<T> {
    type Output = Vec<T::Output>;

    async fn forward(&mut self, diff: Vec<D>) -> object_rainbow::Result<Self::Output> {
        let mut output = Vec::new();
        for diff in diff {
            output.push(self.0.forward(diff).await?);
        }
        Ok(output)
    }
}

impl<T: Forward<D>, D: Send + Traversible> Forward<Point<D>> for Compat<T> {
    type Output = T::Output;

    async fn forward(&mut self, diff: Point<D>) -> object_rainbow::Result<Self::Output> {
        self.0.forward(diff.fetch().await?).await
    }
}

impl<T> Equivalent<T> for Compat<T> {
    fn into_equivalent(self) -> T {
        self.0
    }

    fn from_equivalent(tree: T) -> Self {
        Self(tree)
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
)]
pub struct DiscardHeader<T>(pub T);

assert_impl!(
    impl<T, E> Inline<E> for DiscardHeader<T>
    where
        T: Inline<E>,
        E: 'static + Send + Sync + Clone,
    {
    }
);

impl<T: Forward<D>, D: Send, H: Send> Forward<(H, D)> for DiscardHeader<T> {
    type Output = T::Output;

    fn forward(
        &mut self,
        (_, diff): (H, D),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        self.0.forward(diff)
    }
}

impl<T> Equivalent<T> for DiscardHeader<T> {
    fn into_equivalent(self) -> T {
        self.0
    }

    fn from_equivalent(tree: T) -> Self {
        Self(tree)
    }
}

pub trait MapDiff<Outer: Send>: Send + Sync {
    type Inner: Send;
    fn map(&self, outer: Outer)
    -> impl Send + Future<Output = object_rainbow::Result<Self::Inner>>;
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
)]
pub struct MappedDiff<T, M> {
    tree: T,
    map: M,
}

impl<T, M> MappedDiff<T, M> {
    pub fn tree(&self) -> &T {
        &self.tree
    }
}

impl<T: Forward<M::Inner>, M: MapDiff<Outer>, Outer: Send> Forward<Outer> for MappedDiff<T, M> {
    type Output = T::Output;

    fn forward(
        &mut self,
        outer: Outer,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move { self.tree.forward(self.map.map(outer).await?).await }
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
    Default,
)]
pub struct Sequential<First, Second> {
    first: First,
    second: Second,
}

impl<First, Second> Sequential<First, Second> {
    pub fn first(&self) -> &First {
        &self.first
    }

    pub fn second(&self) -> &Second {
        &self.second
    }
}

impl<Diff: Send + Clone, First: Forward<Diff>, Second: Forward<Diff>> Forward<Diff>
    for Sequential<First, Second>
{
    type Output = Second::Output;
    fn forward(
        &mut self,
        diff: Diff,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move {
            self.first.forward(diff.clone()).await?;
            self.second.forward(diff).await
        }
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
    Default,
)]
pub struct Parallel<A, B> {
    a: A,
    b: B,
}

impl<A, B> Parallel<A, B> {
    pub fn a(&self) -> &A {
        &self.a
    }

    pub fn b(&self) -> &B {
        &self.b
    }
}

impl<Diff: Send + Clone, A: Forward<Diff>, B: Forward<Diff>> Forward<Diff> for Parallel<A, B> {
    type Output = (A::Output, B::Output);

    fn forward(
        &mut self,
        diff: Diff,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move { futures_util::try_join!(self.a.forward(diff.clone()), self.b.forward(diff)) }
    }
}

impl<A, B> Equivalent<Parallel<A, B>> for Sequential<A, B> {
    fn into_equivalent(self) -> Parallel<A, B> {
        let Self {
            first: a,
            second: b,
        } = self;
        Parallel { a, b }
    }

    fn from_equivalent(object: Parallel<A, B>) -> Self {
        object.into_equivalent()
    }
}

impl<A, B> Equivalent<Sequential<A, B>> for Parallel<A, B> {
    fn into_equivalent(self) -> Sequential<A, B> {
        let Self { a, b } = self;
        Sequential {
            first: a,
            second: b,
        }
    }

    fn from_equivalent(object: Sequential<A, B>) -> Self {
        object.into_equivalent()
    }
}
