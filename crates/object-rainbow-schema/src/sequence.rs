use crate::*;

#[derive(Debug, ToOutput, ListHashes, Topological, Parse, PartialEq)]
pub struct SequenceValue {
    pub schema: Extras<Arc<InlineSchema>>,
    pub items: Vec<Arc<InlineValue>>,
}

impl Tagged for SequenceValue {}

impl AbstractValue for SequenceValue {
    type Schema = TailSchema;

    fn schema(&self) -> Self::Schema {
        TailSchema::Sequence(self.schema.0.clone())
    }
}

impl AbstractCollection for SequenceValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items.clone()
    }
}
