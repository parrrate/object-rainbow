use crate::{map_extra::MapExtra, *};

#[derive(
    Debug, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Clone, Copy, Parse, ParseInline,
)]
pub struct Extra0;

impl<A: 'static + Clone, B: 'static + Clone> MapExtra<(A, B)> for Extra0 {
    type Mapped = A;

    fn map_extra(&self, (extra, _): (A, B)) -> Self::Mapped {
        extra
    }
}
