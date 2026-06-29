use object_rainbow::{extra_none_terminated::Ent, none_terminated::Nt};

use crate::*;

#[derive(Debug, Parse, ParseInline, ListHashes, Topological, PartialEq)]
pub struct NtValue {
    pub schema: Extras<Arc<InlineSchema>>,
    pub items: Nt<Vec<Shared<InlineValue>>>,
}

impl ToOutput for NtValue {
    fn to_output(&self, output: &mut impl Output) {
        for item in &self.items {
            item.some_output(output);
        }
        self.schema.none_output(output);
    }
}

impl InlineOutput for NtValue {}
impl Tagged for NtValue {}

impl AbstractValue for NtValue {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Nt(self.schema.0.clone())
    }
}

impl AbstractCollection for NtValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items
            .iter()
            .cloned()
            .map(|Shared(value)| value)
            .collect()
    }
}

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
