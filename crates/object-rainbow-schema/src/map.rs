use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    extra_option::ExtraOption, map_extra::TryMap,
};

use crate::{AsMap, InlineValue, IsUnit, dynamic::InlineDynamic};

#[derive(
    Enum, Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq,
)]
pub enum InlineMap {
    I,
    K1(InlineDynamic),
    K,
}

impl Tagged for InlineMap {}

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

impl AsMap<Arc<InlineMap>> for InlineValue {
    fn as_map(&self) -> object_rainbow::Result<Arc<InlineMap>> {
        match self {
            Self::Option(ExtraOption::Some(value)) => value.as_map(),
            Self::Concat(a, b) if b.is_unit() => a.as_map(),
            Self::Concat(a, b) if a.is_unit() => b.as_map(),
            Self::Array(array)
                if let Some(first) = array.items.first()
                    && array.items.len() == 1
                    && let only = first =>
            {
                only.as_map()
            }
            Self::Enum(value) => value.value.as_map(),
            Self::Map(map) => Ok(map.clone()),
            _ => Err(object_rainbow::error_operation!("not a map")),
        }
    }
}
