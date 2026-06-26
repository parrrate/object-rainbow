#[cfg(feature = "amt")]
use object_rainbow::{
    map_extra::MappedExtra,
    tuple_extra::{Extra0, Extra1},
};
#[cfg(feature = "amt")]
use object_rainbow_amt::AmtMap;
#[cfg(feature = "amt")]
use object_rainbow_point::Point;

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

#[derive(ToOutput, InlineOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum CollectionValue {
    #[cfg(feature = "amt")]
    Amt(Point<AmtMap<MappedExtra<InlineValue, Extra0>, MappedExtra<InlineValue, Extra1>>>),
}

impl Tagged for CollectionValue {}

impl<I: PointInput<Extra = CollectionSchema>> ParseInline<I> for CollectionValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        #[allow(unreachable_code, unused)]
        let schema = input.extra().clone();
        match schema {
            #[cfg(feature = "amt")]
            CollectionSchema::Amt(kv) => Ok(Self::Amt(input.parse_inline_extra(kv)?)),
        }
    }
}
