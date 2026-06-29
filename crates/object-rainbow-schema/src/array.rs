use crate::*;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Tagged,
    Clone,
    PartialEq,
)]
pub struct ArraySchema {
    pub len: u64,
    pub schema: Arc<InlineSchema>,
}

impl AbstractSchema for ArraySchema {
    fn niche(&self) -> SchemaNiche {
        self.schema.niche().repeat(self.len)
    }
}

impl DefaultSchema<ArrayValue> for ArraySchema {
    fn default_value(&self) -> Option<ArrayValue> {
        Some(ArrayValue {
            items: std::iter::repeat_n(self.schema.default_value().map(Arc::new), self.len as _)
                .collect::<Option<_>>()?,
            schema: Extras(self.schema.clone()),
        })
    }
}

impl DefaultIsMin for ArraySchema {
    fn default_is_min(&self) -> bool {
        self.len == 0 || self.schema.default_is_min()
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
        let ArraySchema { len, schema } = input.extra().clone();
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
        ArraySchema {
            len: self.items.len() as _,
            schema: self.schema.0.clone(),
        }
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
