use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, Output, ParseAsInline, ParseInline, PointInput, Tagged, ToOutput,
    Topological,
};
use object_rainbow_point::Point;

use crate::{AbstractValue, InlineSchema, TailSchema, TailValue};

pub struct PointSchema {
    pub schema: Arc<TailSchema>,
}

#[derive(ListHashes, Topological, ParseAsInline)]
pub struct ValuePoint {
    pub point: Point<Arc<TailValue>>,
    pub schema: Arc<TailSchema>,
}

impl ToOutput for ValuePoint {
    fn to_output(&self, output: &mut impl Output) {
        self.point.to_output(output);
    }
}

impl InlineOutput for ValuePoint {}
impl Tagged for ValuePoint {}

impl AbstractValue for ValuePoint {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Point(self.schema.clone())
    }
}

impl<I: PointInput<Extra = Arc<TailSchema>>> ParseInline<I> for ValuePoint {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(Self {
            point: input.parse_inline()?,
            schema,
        })
    }
}
