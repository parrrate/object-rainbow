use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, ToOutput, map_extra::Map};

use crate::InlineValue;

#[derive(Enum, Debug, ToOutput, InlineOutput)]
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
