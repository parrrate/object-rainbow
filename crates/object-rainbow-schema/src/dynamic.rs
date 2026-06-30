use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    inline_extra::InlineExtra, map_extra::MappedExtra, tuple_extra::Extra0,
};

use crate::{InlineSchema, InlineValue};

#[derive(
    Debug, ToOutput, InlineOutput, ListHashes, Topological, Tagged, Parse, ParseInline, PartialEq,
)]
pub struct InlineDynamic(
    pub MappedExtra<MappedExtra<Arc<InlineValue>, Extra0>, InlineExtra<Arc<InlineSchema>>>,
);

impl InlineDynamic {
    pub fn value(&self) -> Arc<InlineValue> {
        self.0.1.1.clone()
    }
}
