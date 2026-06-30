use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, ToOutput, inline_extra::InlineExtra, map_extra::MappedExtra,
    tuple_extra::Extra0,
};

use crate::{InlineSchema, InlineValue};

#[derive(Debug, ToOutput, InlineOutput, ListHashes)]
pub struct InlineDynamic(
    pub MappedExtra<MappedExtra<Arc<InlineSchema>, Extra0>, InlineExtra<Arc<InlineValue>>>,
);
