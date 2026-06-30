use std::sync::Arc;

use object_rainbow::{ToOutput, inline_extra::InlineExtra, map_extra::MappedExtra};

use crate::{InlineSchema, InlineValue};

#[derive(Debug, ToOutput)]
pub struct InlineDynamic(pub MappedExtra<Arc<InlineSchema>, InlineExtra<Arc<InlineValue>>>);
