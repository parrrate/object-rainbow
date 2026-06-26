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

#[cfg(feature = "amt")]
pub type AmtMapSchema = (Arc<InlineSchema>, Arc<InlineSchema>);
#[cfg(feature = "amt")]
pub type AmtSetSchema = Arc<InlineSchema>;

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
            Self::AmtMap(_) => SchemaNiche::point(),
        }
    }
}

#[cfg(feature = "amt")]
pub type AmtMapInner =
    AmtMap<MappedExtra<Arc<InlineValue>, Extra0>, MappedExtra<Arc<InlineValue>, Extra1>>;

#[cfg(feature = "amt")]
#[derive(ListHashes, Topological, Tagged, ParseAsInline)]
pub struct AmtMapValue {
    pub kv: AmtMapSchema,
    pub map: Point<AmtMapInner>,
}

#[cfg(feature = "amt")]
impl ToOutput for AmtMapValue {
    fn to_output(&self, output: &mut impl Output) {
        self.map.to_output(output);
    }
}

#[cfg(feature = "amt")]
impl InlineOutput for AmtMapValue {}

#[cfg(feature = "amt")]
impl<I: PointInput<Extra = AmtMapSchema>> ParseInline<I> for AmtMapValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let kv = input.extra().clone();
        Ok(Self {
            kv,
            map: input.parse_inline()?,
        })
    }
}

#[cfg(feature = "amt")]
impl AbstractValue for AmtMapValue {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::AmtMap(self.kv.clone())
    }
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum CollectionValue {
    #[cfg(feature = "amt")]
    AmtMap(AmtMapValue),
}

impl Tagged for CollectionValue {}

impl<I: PointInput<Extra = CollectionSchema>> ParseInline<I> for CollectionValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        match input.extra().clone() {
            #[cfg(feature = "amt")]
            CollectionSchema::AmtMap(kv) => Ok(Self::AmtMap(input.parse_inline_extra(kv)?)),
        }
    }
}

impl AbstractValue for CollectionValue {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        match *self {
            #[cfg(feature = "amt")]
            Self::AmtMap(ref value) => value.schema(),
        }
    }
}

impl DefaultSchema<CollectionValue> for CollectionSchema {
    fn default_value(&self) -> Option<CollectionValue> {
        match self.clone() {
            #[cfg(feature = "amt")]
            Self::AmtMap(kv) => Some(CollectionValue::AmtMap(AmtMapValue {
                kv,
                map: Default::default(),
            })),
        }
    }
}

impl DefaultIsMin for CollectionSchema {
    fn default_is_min(&self) -> bool {
        match self.clone() {
            #[cfg(feature = "amt")]
            Self::AmtMap(_) => false,
        }
    }
}

impl From<CollectionSchema> for InlineSchema {
    fn from(schema: CollectionSchema) -> Self {
        Self::Collection(schema)
    }
}

impl From<CollectionValue> for InlineValue {
    fn from(value: CollectionValue) -> Self {
        Self::Collection(value)
    }
}
