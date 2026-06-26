use crate::*;

#[derive(Enum, ToOutput, Parse, ParseInline, ListHashes, Topological, Clone)]
pub enum CollectionSchema {
    Amt(
        #[cfg(feature = "amt")] (Arc<InlineSchema>, Arc<InlineSchema>),
        #[cfg(not(feature = "amt"))] Infallible,
    ),
}

impl InlineOutput for CollectionSchema {}
impl Tagged for CollectionSchema {}

pub enum CollectionValue {}
