use std::sync::Arc;

use object_rainbow::map_extra::Map;

use crate::InlineValue;

pub enum InlineMap {
    I,
}

impl Map<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> Self::Mapped {
        match self {
            Self::I => value,
        }
    }
}
