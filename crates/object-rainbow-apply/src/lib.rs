use core::future::ready;

use futures_util::future::try_join;
use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, derive_for_wrapped,
    map_extra::{SmExtra, StaticMap},
    tuple_extra::ToTuple2,
};

#[cfg(feature = "amt")]
mod amt;
#[cfg(feature = "hamt")]
mod hamt;
#[cfg(feature = "point")]
mod point;
#[cfg(feature = "trie")]
mod trie;

#[derive_for_wrapped]
pub trait Apply<Diff: Send>: Send {
    type Output: Send;
    /// Must stay isomorphic under [`Equivalent`] conversions.
    fn apply(
        &mut self,
        diff: Diff,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>>;
}

impl Apply<()> for () {
    type Output = ();

    fn apply(
        &mut self,
        (): (),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        ready(Ok(()))
    }
}

impl<A: Apply<DiffA>, B: Apply<DiffB>, DiffA: Send, DiffB: Send> Apply<(DiffA, DiffB)> for (A, B) {
    type Output = (A::Output, B::Output);

    fn apply(
        &mut self,
        (a, b): (DiffA, DiffB),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        try_join(self.0.apply(a), self.1.apply(b))
    }
}

impl<M: Send + StaticMap<D, Mapped: Send>, D: Send> Apply<D> for SmExtra<M> {
    type Output = M::Mapped;

    fn apply(&mut self, d: D) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        core::future::ready(Ok(M::static_map(d)))
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

impl<Diff: Send, First: Apply<Diff>, Second: Apply<First::Output>> Apply<Diff>
    for Sequential<First, Second>
{
    type Output = Second::Output;

    async fn apply(&mut self, diff: Diff) -> object_rainbow::Result<Self::Output> {
        self.second.apply(self.first.apply(diff).await?).await
    }
}

pub type Parallel<A, B> = Sequential<ToTuple2, (A, B)>;

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
