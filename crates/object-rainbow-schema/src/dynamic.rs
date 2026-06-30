use std::sync::Arc;

use object_rainbow::{inline_extra::InlineExtra, map_extra::MappedExtra};

use crate::{InlineSchema, InlineValue};

#[derive(Debug)]
pub struct InlineDynamic(pub MappedExtra<Arc<InlineSchema>, InlineExtra<Arc<InlineValue>>>);
