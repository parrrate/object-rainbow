use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput, Topological,
};
use object_rainbow_point::{Extras, IntoPoint, Point};

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
)]
pub struct PointSchema {
    pub schema: Arc<TailSchema>,
}

impl AbstractSchema for PointSchema {
    fn niche(&self) -> SchemaNiche {
        SchemaNiche::HashNiche(u128::MAX)
    }
}

impl DefaultSchema<ValuePoint> for PointSchema {
    fn default_value(&self) -> Option<ValuePoint> {
        Some(ValuePoint {
            schema: Extras(self.schema.clone()),
            point: Arc::new(self.schema.default_value()?).point(),
        })
    }
}

impl From<PointSchema> for InlineSchema {
    fn from(schema: PointSchema) -> Self {
        Self::Point(schema)
    }
}

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline)]
pub struct ValuePoint {
    pub schema: Extras<Arc<TailSchema>>,
    pub point: Point<Arc<TailValue>>,
}

impl Tagged for ValuePoint {}

impl AbstractValue for ValuePoint {
    type Schema = PointSchema;

    fn schema(&self) -> Self::Schema {
        PointSchema {
            schema: self.schema.0.clone(),
        }
    }
}

impl From<ValuePoint> for InlineValue {
    fn from(value: ValuePoint) -> Self {
        Self::Point(value)
    }
}
