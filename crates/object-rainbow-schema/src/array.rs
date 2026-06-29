use object_rainbow::extra_array::ExtraArray;

use crate::*;

pub type ArraySchema = (u64, Arc<InlineSchema>);

impl AbstractSchema for ArraySchema {
    fn niche(&self) -> SchemaNiche {
        self.1.niche().repeat(self.0)
    }
}

impl DefaultSchema<ArrayValue> for ArraySchema {
    fn default_value(&self) -> Option<ArrayValue> {
        Some(ArrayValue {
            schema: MappedExtra(Default::default(), Extras(self.1.clone())),
            items: std::iter::repeat_n(self.1.default_value().map(Arc::new), self.0 as _)
                .collect::<Option<_>>()?,
        })
    }
}

impl DefaultIsMin for ArraySchema {
    fn default_is_min(&self) -> bool {
        self.0 == 0 || self.1.default_is_min()
    }
}

impl SizeSchema for ArraySchema {
    fn size(&self) -> Option<u64> {
        self.0.checked_mul(self.1.size()?)
    }
}

impl From<ArraySchema> for InlineSchema {
    fn from(schema: ArraySchema) -> Self {
        Self::Array(schema)
    }
}

#[derive(Debug, ToOutput, ParseAsInline, ParseInline, ListHashes, Topological, PartialEq)]
pub struct ArrayValue {
    pub schema: MappedExtra<Extras<Arc<InlineSchema>>, Extra1>,
    pub items: ExtraArray<Arc<InlineValue>>,
}

impl InlineOutput for ArrayValue {}
impl Tagged for ArrayValue {}

impl AbstractValue for ArrayValue {
    type Schema = ArraySchema;

    fn schema(&self) -> Self::Schema {
        (self.items.len() as _, self.schema.1.0.clone())
    }
}

impl AbstractCollection for ArrayValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items.clone()
    }
}

impl From<ArrayValue> for InlineValue {
    fn from(value: ArrayValue) -> Self {
        Self::Array(value)
    }
}
