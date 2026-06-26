#[cfg(feature = "_collections-kv")]
use object_rainbow::{
    map_extra::MappedExtra,
    tuple_extra::{Extra0, Extra1},
};
#[cfg(feature = "amt")]
use object_rainbow_amt::{AmtMap, AmtSet};
#[cfg(feature = "hamt")]
use object_rainbow_hamt::{HamtMap, HamtSet};
use object_rainbow_point::Extras;

use crate::*;

#[cfg(feature = "_collections-kv")]
pub type KvSchema = (Arc<InlineSchema>, Arc<InlineSchema>);
#[cfg(feature = "_collections-item")]
pub type ItemSchema = Arc<InlineSchema>;

#[derive(Debug, Enum, ToOutput, Parse, ParseInline, ListHashes, Topological, Clone)]
pub enum CollectionSchema {
    AmtMap(
        #[cfg(feature = "amt")] KvSchema,
        #[cfg(not(feature = "amt"))] Infallible,
    ),
    AmtSet(
        #[cfg(feature = "amt")] ItemSchema,
        #[cfg(not(feature = "amt"))] Infallible,
    ),
    HamtMap(
        #[cfg(feature = "hamt")] ItemSchema,
        #[cfg(not(feature = "hamt"))] Infallible,
    ),
    HamtSet(
        #[cfg(feature = "hamt")] (),
        #[cfg(not(feature = "hamt"))] Infallible,
    ),
}

impl InlineOutput for CollectionSchema {}
impl Tagged for CollectionSchema {}

impl AbstractSchema for CollectionSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::AmtMap(_) => SchemaNiche::point(),
            Self::AmtSet(_) => SchemaNiche::point(),
            Self::HamtMap(_) => SchemaNiche::point(),
            Self::HamtSet(_) => SchemaNiche::point(),
        }
    }
}

#[cfg(feature = "amt")]
pub type AmtMapInner =
    AmtMap<MappedExtra<Arc<InlineValue>, Extra0>, MappedExtra<Arc<InlineValue>, Extra1>>;
#[cfg(feature = "amt")]
pub type AmtSetInner = AmtSet<Arc<InlineValue>>;
#[cfg(feature = "hamt")]
pub type HamtMapInner = HamtMap<Arc<InlineValue>>;

#[cfg(feature = "_collections-kv")]
#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Tagged, Parse, ParseInline)]
pub struct KvValue<T> {
    pub kv: Extras<KvSchema>,
    pub map: T,
}
#[cfg(feature = "_collections-item")]
#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Tagged, Parse, ParseInline)]
pub struct ItemValue<T> {
    pub item: Extras<ItemSchema>,
    pub set: T,
}

#[cfg(feature = "_collections-kv")]
impl<T: Default> KvValue<T> {
    pub fn schema_default(kv: KvSchema) -> Self {
        Self {
            kv: Extras(kv),
            map: Default::default(),
        }
    }
}
#[cfg(feature = "_collections-item")]
impl<T: Default> ItemValue<T> {
    pub fn schema_default(item: ItemSchema) -> Self {
        Self {
            item: Extras(item),
            set: Default::default(),
        }
    }
}

#[cfg(feature = "amt")]
impl AbstractValue for KvValue<object_rainbow_point::Point<AmtMapInner>> {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::AmtMap(self.kv.0.clone())
    }
}
#[cfg(feature = "amt")]
impl AbstractValue for ItemValue<object_rainbow_point::Point<AmtSetInner>> {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::AmtSet(self.item.0.clone())
    }
}
#[cfg(feature = "hamt")]
impl AbstractValue for ItemValue<HamtMapInner> {
    type Schema = CollectionSchema;

    fn schema(&self) -> Self::Schema {
        CollectionSchema::HamtMap(self.item.0.clone())
    }
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum CollectionValue {
    #[cfg(feature = "amt")]
    AmtMap(KvValue<object_rainbow_point::Point<AmtMapInner>>),
    #[cfg(feature = "amt")]
    AmtSet(ItemValue<object_rainbow_point::Point<AmtSetInner>>),
    #[cfg(feature = "hamt")]
    HamtMap(ItemValue<HamtMapInner>),
    #[cfg(feature = "hamt")]
    HamtSet(HamtSet),
}

impl Tagged for CollectionValue {}

impl<I: PointInput<Extra = CollectionSchema>> ParseInline<I> for CollectionValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        match input.extra().clone() {
            #[cfg(feature = "amt")]
            CollectionSchema::AmtMap(kv) => Ok(Self::AmtMap(input.parse_inline_extra(kv)?)),
            #[cfg(feature = "amt")]
            CollectionSchema::AmtSet(item) => Ok(Self::AmtSet(input.parse_inline_extra(item)?)),
            #[cfg(feature = "hamt")]
            CollectionSchema::HamtMap(item) => Ok(Self::HamtMap(input.parse_inline_extra(item)?)),
            #[cfg(feature = "hamt")]
            CollectionSchema::HamtSet(()) => Ok(Self::HamtSet(input.parse_inline()?)),
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
            #[cfg(feature = "hamt")]
            Self::HamtMap(ref value) => value.schema(),
            #[cfg(feature = "hamt")]
            Self::HamtSet(_) => CollectionSchema::HamtSet(()),
        }
    }
}

impl DefaultSchema<CollectionValue> for CollectionSchema {
    fn default_value(&self) -> Option<CollectionValue> {
        match self.clone() {
            #[cfg(feature = "amt")]
            Self::AmtMap(kv) => Some(CollectionValue::AmtMap(KvValue::schema_default(kv))),
            #[cfg(feature = "amt")]
            Self::AmtSet(item) => Some(CollectionValue::AmtSet(ItemValue::schema_default(item))),
            #[cfg(feature = "hamt")]
            Self::HamtMap(item) => Some(CollectionValue::HamtMap(ItemValue::schema_default(item))),
            #[cfg(feature = "hamt")]
            Self::HamtSet(()) => Some(CollectionValue::HamtSet(Default::default())),
        }
    }
}

impl DefaultIsMin for CollectionSchema {
    fn default_is_min(&self) -> bool {
        match self {
            Self::AmtMap(_) => false,
            Self::AmtSet(_) => false,
            Self::HamtMap(_) => false,
            Self::HamtSet(_) => false,
        }
    }
}

impl SizeSchema for CollectionSchema {
    fn size(&self) -> Option<u64> {
        match self {
            Self::AmtMap(_) => Some(32),
            Self::AmtSet(_) => Some(32),
            Self::HamtMap(_) => Some(32),
            Self::HamtSet(_) => Some(32),
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
