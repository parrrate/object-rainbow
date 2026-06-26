#[cfg(feature = "_collections-map")]
use object_rainbow::{
    map_extra::MappedExtra,
    tuple_extra::{Extra0, Extra1},
};
#[cfg(feature = "amt")]
use object_rainbow_amt::{AmtMap, AmtSet};
#[cfg(feature = "amt")]
use object_rainbow_point::Extras;
use object_rainbow_point::Point;

use crate::*;

#[cfg(feature = "_collections-map")]
pub type KvSchema = (Arc<InlineSchema>, Arc<InlineSchema>);
#[cfg(feature = "_collections-set")]
pub type ItemSchema = Arc<InlineSchema>;

#[derive(Enum, ToOutput, Parse, ParseInline, ListHashes, Topological, Clone)]
pub enum CollectionSchema {
    AmtMap(
        #[cfg(feature = "amt")] KvSchema,
        #[cfg(not(feature = "amt"))] Infallible,
    ),
    AmtSet(
        #[cfg(feature = "amt")] ItemSchema,
        #[cfg(not(feature = "amt"))] Infallible,
    ),
}

impl InlineOutput for CollectionSchema {}
impl Tagged for CollectionSchema {}

impl AbstractSchema for CollectionSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::AmtMap(_) => SchemaNiche::point(),
            Self::AmtSet(_) => SchemaNiche::point(),
        }
    }
}

#[cfg(feature = "amt")]
pub type AmtMapInner =
    AmtMap<MappedExtra<Arc<InlineValue>, Extra0>, MappedExtra<Arc<InlineValue>, Extra1>>;
#[cfg(feature = "amt")]
pub type AmtSetInner = AmtSet<Arc<InlineValue>>;

#[cfg(feature = "amt")]
#[derive(ToOutput, InlineOutput, ListHashes, Topological, Tagged, Parse, ParseInline)]
pub struct AmtMapValue {
    pub kv: Extras<KvSchema>,
    pub map: Point<AmtMapInner>,
}
#[cfg(feature = "amt")]
#[derive(ToOutput, ListHashes, Topological, Tagged, Parse, ParseInline)]
pub struct AmtSetValue {
    pub item: Extras<ItemSchema>,
    pub set: Point<AmtSetInner>,
}

#[cfg(feature = "amt")]
impl InlineOutput for AmtSetValue {}

#[cfg(feature = "amt")]
impl AbstractValue for AmtMapValue {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::AmtMap(self.kv.0.clone())
    }
}
#[cfg(feature = "amt")]
impl AbstractValue for AmtSetValue {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::AmtSet(self.item.0.clone())
    }
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum CollectionValue {
    #[cfg(feature = "amt")]
    AmtMap(AmtMapValue),
    #[cfg(feature = "amt")]
    AmtSet(AmtSetValue),
}

impl Tagged for CollectionValue {}

impl<I: PointInput<Extra = CollectionSchema>> ParseInline<I> for CollectionValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        match input.extra().clone() {
            #[cfg(feature = "amt")]
            CollectionSchema::AmtMap(kv) => Ok(Self::AmtMap(input.parse_inline_extra(kv)?)),
            #[cfg(feature = "amt")]
            CollectionSchema::AmtSet(item) => Ok(Self::AmtSet(input.parse_inline_extra(item)?)),
        }
    }
}

impl AbstractValue for CollectionValue {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        match *self {
            #[cfg(feature = "amt")]
            Self::AmtMap(ref value) => value.schema(),
            #[cfg(feature = "amt")]
            Self::AmtSet(ref value) => value.schema(),
        }
    }
}

impl DefaultSchema<CollectionValue> for CollectionSchema {
    fn default_value(&self) -> Option<CollectionValue> {
        match self.clone() {
            #[cfg(feature = "amt")]
            Self::AmtMap(kv) => Some(CollectionValue::AmtMap(AmtMapValue {
                kv: Extras(kv),
                map: Default::default(),
            })),
            #[cfg(feature = "amt")]
            Self::AmtSet(item) => Some(CollectionValue::AmtSet(AmtSetValue {
                item: Extras(item),
                set: Default::default(),
            })),
        }
    }
}

impl DefaultIsMin for CollectionSchema {
    fn default_is_min(&self) -> bool {
        match self.clone() {
            #[cfg(feature = "amt")]
            Self::AmtMap(_) => false,
            #[cfg(feature = "amt")]
            Self::AmtSet(_) => false,
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
