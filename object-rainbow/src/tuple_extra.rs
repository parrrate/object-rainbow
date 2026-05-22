use crate::map_extra::{SmExtra, StaticMap};

pub struct StaticToTuple0;

impl<T> StaticMap<T> for StaticToTuple0 {
    type Mapped = ();

    fn map_extra(_: T) -> Self::Mapped {}
}

pub type ToTuple0 = SmExtra<StaticToTuple0>;

pub struct StaticExtra0;

impl<A, B> StaticMap<(A, B)> for StaticExtra0 {
    type Mapped = A;

    fn map_extra((a, _): (A, B)) -> Self::Mapped {
        a
    }
}

pub type Extra0 = SmExtra<StaticExtra0>;

pub struct StaticExtra1;

impl<A, B> StaticMap<(A, B)> for StaticExtra1 {
    type Mapped = B;

    fn map_extra((_, b): (A, B)) -> Self::Mapped {
        b
    }
}

pub type Extra1 = SmExtra<StaticExtra1>;

pub struct StaticSwap;

impl<A, B> StaticMap<(A, B)> for StaticSwap {
    type Mapped = (B, A);

    fn map_extra((a, b): (A, B)) -> Self::Mapped {
        (b, a)
    }
}

pub type Swap = SmExtra<StaticSwap>;
