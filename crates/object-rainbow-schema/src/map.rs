use std::sync::Arc;

use object_rainbow::map_extra::Map;

use crate::InlineValue;

#[derive(Debug)]
pub enum InlineMap {
    I,
    K1(Arc<InlineValue>),
}

impl Map<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> Self::Mapped {
        match self {
            Self::I => value,
            Self::K1(value) => value.clone(),
        }
    }
}
