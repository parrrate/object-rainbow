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
            schema: self.schema.clone(),
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

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct ArrayValue {
    pub schema: Arc<InlineSchema>,
    pub items: Vec<Arc<InlineValue>>,
}
