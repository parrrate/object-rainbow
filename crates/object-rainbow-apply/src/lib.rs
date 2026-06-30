use core::future::ready;

use futures_util::future::try_join;
use object_rainbow::{
    derive_for_wrapped,
    map_extra::{SmExtra, StaticMap},
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
