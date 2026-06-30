use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, ToOutput, Topological, map_extra::Map,
};

use crate::{InlineValue, dynamic::InlineDynamic};

#[derive(
    Enum, Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq,
)]
pub enum InlineMap {
    I,
    K1(InlineDynamic),
}

impl Map<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> Self::Mapped {
        match self {
            Self::I => value,
            Self::K1(value) => value.value(),
        }
    }
}
