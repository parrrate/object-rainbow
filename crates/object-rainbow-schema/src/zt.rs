use object_rainbow::{CanonicalExtra, zero_terminated::Zt};

use crate::*;

pub type ZtValue = (Extras<Arc<TailSchema>>, Zt<Arc<TailValue>>);

impl AbstractValue for ZtValue {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Zt(self.canonical_extra())
    }
}

impl AbstractCollection for ZtValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.1.items()
    }
}

impl From<ZtValue> for InlineValue {
    fn from(value: ZtValue) -> Self {
        Self::Zt(value)
    }
}

pub fn zt_schema_default(schema: Arc<TailSchema>) -> Option<ZtValue> {
    let value = Zt::new(Arc::new(schema.default_value()?)).ok()?;
    Some((Extras(schema), value))
}
