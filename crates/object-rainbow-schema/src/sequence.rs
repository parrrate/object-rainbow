use object_rainbow::CanonicalExtra;

use crate::*;

pub type SequenceValue = (Extras<Arc<InlineSchema>>, Vec<Arc<InlineValue>>);

impl AbstractValue for SequenceValue {
    type Schema = TailSchema;

    fn schema(&self) -> Self::Schema {
        TailSchema::Sequence(self.canonical_extra())
    }
}

impl AbstractCollection for SequenceValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.1.clone()
    }
}
