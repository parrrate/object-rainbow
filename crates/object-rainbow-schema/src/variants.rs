use crate::*;

#[derive(
    Debug, ToOutput, InlineOutput, Parse, ParseInline, ListHashes, Topological, Tagged, PartialEq,
)]
pub struct EnumSchema<T> {
    pub kind: NumericSchema,
    pub variants: Arc<LpVec<Arc<T>>>,
}

impl<T> Clone for EnumSchema<T> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl<T: AbstractSchema> AbstractSchema for EnumSchema<T> {
    fn niche(&self) -> SchemaNiche {
        self.kind.niche().stop()
    }
}

impl<T: AbstractValue<Schema: DefaultSchema<T>>> DefaultSchema<EnumValue<T>>
    for EnumSchema<T::Schema>
{
    fn default_value(&self) -> Option<EnumValue<T>> {
        Some(EnumValue {
            kind: self.kind.default_value()?,
            variants: self.variants.clone(),
            value: Arc::new(self.variants.first()?.default_value()?),
        })
    }
}

impl<T: DefaultIsMin> DefaultIsMin for EnumSchema<T> {
    fn default_is_min(&self) -> bool {
        self.kind.default_is_min()
            && self
                .variants
                .first()
                .is_some_and(|schema| schema.default_is_min())
    }
}

impl From<EnumSchema<InlineSchema>> for InlineSchema {
    fn from(schema: EnumSchema<InlineSchema>) -> Self {
        Self::Enum(schema)
    }
}

impl From<EnumSchema<TailSchema>> for TailSchema {
    fn from(schema: EnumSchema<TailSchema>) -> Self {
        Self::Enum(schema)
    }
}
