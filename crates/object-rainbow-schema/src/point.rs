use std::sync::Arc;

use object_rainbow::{
    CanonicalExtra, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput,
    Topological,
};
use object_rainbow_point::{ExtraPoint, Extras, IntoPoint};

use crate::{
    AbstractSchema, AbstractValue, DefaultSchema, InlineSchema, InlineValue, SchemaNiche,
    TailSchema, TailValue,
};

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Tagged,
    Clone,
    PartialEq,
)]
pub struct PointSchema {
    pub schema: Arc<TailSchema>,
}

impl AbstractSchema for PointSchema {
    fn niche(&self) -> SchemaNiche {
        SchemaNiche::point()
    }
}

impl DefaultSchema<ValuePoint> for PointSchema {
    fn default_value(&self) -> Option<ValuePoint> {
        Some(ValuePoint {
            extra: Extras(self.schema.clone()),
            point: Arc::new(self.schema.default_value()?).point(),
        })
    }
}

impl From<PointSchema> for InlineSchema {
    fn from(schema: PointSchema) -> Self {
        Self::Point(schema)
    }
}

pub type ValuePoint = ExtraPoint<Arc<TailValue>, Arc<TailSchema>>;

impl AbstractValue for ValuePoint {
    type Schema = PointSchema;

    fn schema(&self) -> Self::Schema {
        PointSchema {
            schema: self.extra.canonical_extra(),
        }
    }
}

impl From<ValuePoint> for InlineValue {
    fn from(value: ValuePoint) -> Self {
        Self::Point(value)
    }
}
