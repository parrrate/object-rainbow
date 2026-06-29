use crate::*;

#[derive(
    Debug, ToOutput, InlineOutput, Parse, ParseInline, ListHashes, Topological, Tagged, PartialEq,
)]
pub struct EnumSchema<T> {
    pub kind: NumericSchema,
    pub variants: Arc<LpVec<Arc<T>>>,
}
