use crate::{map_extra::Map, *};

/// Parses `Extra`, then provides it to `T`'s parser.
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
    PartialEq,
)]
pub struct InlineExtra<Extra = ()>(pub Extra);

impl<E: 'static + Clone, X: 'static + Clone> Map<X> for InlineExtra<E> {
    type Mapped = (E, X);

    fn map(&self, extra: X) -> Self::Mapped {
        (self.0.clone(), extra)
    }
}
