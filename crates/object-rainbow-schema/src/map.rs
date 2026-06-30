use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, ToOutput, Topological,
    inline_extra::InlineExtra,
    map_extra::{Map, MappedExtra},
};

use crate::{InlineSchema, InlineValue};

#[derive(
    Enum, Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq,
)]
pub enum InlineMap {
    I,
    K1(MappedExtra<Arc<InlineSchema>, InlineExtra<Arc<InlineValue>>>),
}

impl Map<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> Self::Mapped {
        match self {
            Self::I => value,
            Self::K1(value) => value.0.0.clone(),
        }
    }
}
