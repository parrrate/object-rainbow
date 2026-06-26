#[cfg(feature = "amt")]
use object_rainbow::{
    map_extra::MappedExtra,
    tuple_extra::{Extra0, Extra1},
};
#[cfg(feature = "amt")]
use object_rainbow_amt::AmtMap;

use crate::*;

#[cfg(feature = "amt")]
pub type AmtMapSchema = (Arc<InlineSchema>, Arc<InlineSchema>);

#[derive(Enum, ToOutput, Parse, ParseInline, ListHashes, Topological, Clone)]
pub enum CollectionSchema {
    AmtMap(
        #[cfg(feature = "amt")] AmtMapSchema,
        #[cfg(not(feature = "amt"))] Infallible,
    ),
}

impl InlineOutput for CollectionSchema {}
impl Tagged for CollectionSchema {}

impl AbstractSchema for CollectionSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::AmtMap(_) => SchemaNiche::Cut,
        }
    }
}

#[cfg(feature = "amt")]
#[derive(ListHashes, Topological, Tagged, ParseAsInline)]
pub struct AmtMapValue {
    pub kv: AmtMapSchema,
    pub map: AmtMap<MappedExtra<InlineValue, Extra0>, MappedExtra<InlineValue, Extra1>>,
}

#[cfg(feature = "amt")]
impl ToOutput for AmtMapValue {
    fn to_output(&self, output: &mut impl Output) {
        self.map.to_output(output);
    }
}

#[cfg(feature = "amt")]
impl InlineOutput for AmtMapValue {}

impl<I: PointInput<Extra = AmtMapSchema>> ParseInline<I> for AmtMapValue
where
    InlineValue: ParseInline<I::WithExtra<Arc<InlineSchema>>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let kv = input.extra().clone();
        Ok(Self {
            kv,
            map: input.parse_inline()?,
        })
    }
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum CollectionValue {
    #[cfg(feature = "amt")]
    AmtMap(AmtMapValue),
}

impl Tagged for CollectionValue {}

impl<I: PointInput<Extra = CollectionSchema>> ParseInline<I> for CollectionValue
where
    AmtMapValue: ParseInline<I::WithExtra<AmtMapSchema>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        #[allow(unreachable_code, unused)]
        let schema = input.extra().clone();
        match schema {
            #[cfg(feature = "amt")]
            CollectionSchema::AmtMap(kv) => Ok(Self::AmtMap(input.parse_inline_extra(kv)?)),
        }
    }
}
