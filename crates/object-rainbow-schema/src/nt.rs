use object_rainbow::extra_none_terminated::Ent;

use crate::*;

pub type NtValue = Ent<Vec<Shared<InlineValue>>, Arc<InlineSchema>>;

impl AbstractValue for Ent<Vec<Shared<InlineValue>>, Arc<InlineSchema>> {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Nt(self.extra.0.clone())
    }
}

impl AbstractCollection for Ent<Vec<Shared<InlineValue>>, Arc<InlineSchema>> {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items
            .iter()
            .cloned()
            .map(|Shared(value)| value)
            .collect()
    }
}
