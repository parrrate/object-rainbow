use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq)]
pub struct ZtValue {
    pub schema: Extras<Arc<TailSchema>>,
    pub value: Zt<Arc<TailValue>>,
}

impl AbstractValue for ZtValue {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Zt(self.schema.0.clone())
    }
}
