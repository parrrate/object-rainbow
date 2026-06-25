use crate::*;

#[derive(Enum)]
pub enum CollectionSchema {
    Amt(
        #[cfg(feature = "amt")] (Arc<InlineSchema>, Arc<InlineSchema>),
        #[cfg(not(feature = "amt"))] Infallible,
    ),
}
