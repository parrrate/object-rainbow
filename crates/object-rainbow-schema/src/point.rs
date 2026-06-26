use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseAsInline, ParseInline, PointInput,
    Tagged, ToOutput, Topological,
};
use object_rainbow_point::{IntoPoint, Point};

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
            point: Arc::new(self.schema.default_value()?).point(),
            schema: self.schema.clone(),
        })
    }
}

impl From<PointSchema> for InlineSchema {
    fn from(schema: PointSchema) -> Self {
        Self::Point(schema)
    }
}

#[derive(ListHashes, Topological, ParseAsInline)]
pub struct ValuePoint {
    pub schema: Arc<TailSchema>,
    pub point: Point<Arc<TailValue>>,
}

impl ToOutput for ValuePoint {
    fn to_output(&self, output: &mut impl Output) {
        self.point.to_output(output);
    }
}

impl InlineOutput for ValuePoint {}
impl Tagged for ValuePoint {}

impl AbstractValue for ValuePoint {
    type Schema = PointSchema;

    fn schema(&self) -> Self::Schema {
        PointSchema {
            schema: self.schema.clone(),
        }
    }
}

impl<I: PointInput<Extra = PointSchema>> ParseInline<I> for ValuePoint {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone().schema;
        Ok(Self {
            point: input.parse_inline_extra(schema.clone())?,
            schema,
        })
    }
}

impl From<ValuePoint> for InlineValue {
    fn from(value: ValuePoint) -> Self {
        Self::Point(value)
    }
}
