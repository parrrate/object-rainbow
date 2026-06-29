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
            items: std::iter::repeat_n(self.1.default_value().map(Arc::new), self.0 as _)
                .collect::<Option<_>>()?,
            schema: Extras(self.1.clone()),
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

#[derive(Debug, ToOutput, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct ArrayValue {
    pub schema: Extras<Arc<InlineSchema>>,
    pub items: Vec<Arc<InlineValue>>,
}

impl InlineOutput for ArrayValue {}
impl Tagged for ArrayValue {}

impl<I: PointInput<Extra = ArraySchema>> ParseInline<I> for ArrayValue
where
    InlineValue: ParseInline<I::WithExtra<Arc<InlineSchema>>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let (len, schema) = input.extra().clone();
        let mut items = Vec::new();
        for _ in 0..len {
            items.push(input.parse_inline_extra(schema.clone())?);
        }
        Ok(Self {
            schema: Extras(schema),
            items,
        })
    }
}

impl AbstractValue for ArrayValue {
    type Schema = ArraySchema;

    fn schema(&self) -> Self::Schema {
        (self.items.len() as _, self.schema.0.clone())
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
