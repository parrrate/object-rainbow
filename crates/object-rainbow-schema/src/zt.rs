use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq)]
pub struct ZtValue {
    pub schema: Extras<Arc<TailSchema>>,
    pub value: Zt<Arc<TailValue>>,
}

impl Tagged for ZtValue {}

impl AbstractValue for ZtValue {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Zt(self.schema.0.clone())
    }
}

impl AbstractCollection for ZtValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.value.items()
    }
}

impl From<ZtValue> for InlineValue {
    fn from(value: ZtValue) -> Self {
        Self::Zt(value)
    }
}

impl ZtValue {
    pub fn schema_default(schema: Arc<TailSchema>) -> Option<Self> {
        let value = Zt::new(Arc::new(schema.default_value()?)).ok()?;
        Some(Self {
            schema: Extras(schema),
            value,
        })
    }
}
