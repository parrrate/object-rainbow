use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, ToOutput, Topological, map_extra::TryMap,
};

use crate::{InlineValue, dynamic::InlineDynamic};

#[derive(
    Enum, Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq,
)]
pub enum InlineMap {
    I,
    K1(InlineDynamic),
    K,
}

impl TryMap<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> object_rainbow::Result<Self::Mapped> {
        Ok(match self {
            Self::I => value,
            Self::K1(value) => value.value(),
            Self::K => Arc::new(InlineValue::Map(Arc::new(Self::K1(InlineDynamic::new(
                value,
            ))))),
        })
    }
}
