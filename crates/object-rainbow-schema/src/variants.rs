use crate::*;

#[derive(
    Debug, ToOutput, InlineOutput, Parse, ParseInline, ListHashes, Topological, Tagged, PartialEq,
)]
pub struct EnumSchema<T> {
    pub kind: NumericSchema,
    pub variants: Arc<LpVec<Arc<T>>>,
}

impl From<EnumSchema<InlineSchema>> for InlineSchema {
    fn from(schema: EnumSchema<InlineSchema>) -> Self {
        Self::Enum(schema)
    }
}
