use crate::map_extra::{SmExtra, StaticMap};

pub struct StaticToTuple0;

impl<T> StaticMap<T> for StaticToTuple0 {
    type Mapped = ();

    fn static_map(_: T) -> Self::Mapped {}
}

pub type ToTuple0 = SmExtra<StaticToTuple0>;

pub struct StaticToTuple2;

impl<T: Clone> StaticMap<T> for StaticToTuple2 {
    type Mapped = (T, T);

    fn static_map(x: T) -> Self::Mapped {
        (x.clone(), x)
    }
}

pub type ToTuple2 = SmExtra<StaticToTuple2>;

pub struct StaticExtra0;

impl<A, B> StaticMap<(A, B)> for StaticExtra0 {
    type Mapped = A;

    fn static_map((a, _): (A, B)) -> Self::Mapped {
        a
    }
}

pub type Extra0 = SmExtra<StaticExtra0>;

pub struct StaticExtra1;

impl<A, B> StaticMap<(A, B)> for StaticExtra1 {
    type Mapped = B;

    fn static_map((_, b): (A, B)) -> Self::Mapped {
        b
    }
}

pub type Extra1 = SmExtra<StaticExtra1>;

pub struct StaticSwap;

impl<A, B> StaticMap<(A, B)> for StaticSwap {
    type Mapped = (B, A);

    fn static_map((a, b): (A, B)) -> Self::Mapped {
        (b, a)
    }
}

pub type Swap = SmExtra<StaticSwap>;

pub struct StaticOneCrossN;

impl<A: Clone, B, T: IntoIterator<Item = B>> StaticMap<(A, T)> for StaticOneCrossN {
    type Mapped = Vec<(A, B)>;

    fn static_map((a, b): (A, T)) -> Self::Mapped {
        b.into_iter().map(|b| (a.clone(), b)).collect()
    }
}

pub type OneCrossN = SmExtra<StaticOneCrossN>;

#[allow(clippy::repr_packed_without_abi)]
mod private {
    use ghost::phantom;

    #[phantom]
    pub struct StaticMap1<M>;
}
#[doc(hidden)]
pub use self::private::*;

pub type StaticMap1<M> = private::StaticMap1<M>;

impl<A, B, M: StaticMap<B>> StaticMap<(A, B)> for StaticMap1<M> {
    type Mapped = (A, M::Mapped);

    fn static_map((a, b): (A, B)) -> Self::Mapped {
        (a, M::static_map(b))
    }
}

pub type Map1<M> = SmExtra<StaticMap1<M>>;
