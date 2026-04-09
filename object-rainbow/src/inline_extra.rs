use crate::{map_extra::MapExtra, *};

/// Parses `Extra`, then provides it to `T`'s parser.
#[derive(Debug, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Clone, Copy, Parse)]
pub struct InlineExtra<Extra = ()>(pub Extra);

impl<E: 'static + Clone, X: 'static + Clone> MapExtra<X> for InlineExtra<E> {
    type Mapped = (E, X);

    fn map_extra(&self, extra: X) -> Self::Mapped {
        (self.0.clone(), extra)
    }
}
