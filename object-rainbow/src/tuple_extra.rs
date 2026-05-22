use crate::{
    map_extra::{MapExtra, SmExtra, StaticMap},
    *,
};

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

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Parse,
    ParseInline,
    Default,
    PartialEq,
    Eq,
)]
pub struct Swap;

impl<A: 'static + Clone, B: 'static + Clone> MapExtra<(A, B)> for Swap {
    type Mapped = (B, A);

    fn map_extra(&self, (a, b): (A, B)) -> Self::Mapped {
        (b, a)
    }
}
