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
            variants: Extras(self.variants.clone()),
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

impl<T: SizeSchema> SizeSchema for EnumSchema<T> {
    fn size(&self) -> Option<u64> {
        let size = self.variants.first()?.size()?;
        for schema in &self.variants[1..] {
            if schema.size()? != size {
                return None;
            }
        }
        self.kind.size()?.checked_add(size)
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

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Tagged, PartialEq)]
pub struct EnumValue<T: AbstractValue> {
    pub kind: NumericValue,
    pub variants: Extras<Arc<LpVec<Arc<T::Schema>>>>,
    pub value: Arc<T>,
}

impl<
    T: AbstractValue + Parse<I::WithExtra<Arc<T::Schema>>>,
    I: PointInput<Extra = EnumSchema<T::Schema>>,
> Parse<I> for EnumValue<T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let EnumSchema { kind, variants } = input.extra().clone();
        let kind: NumericValue = input.parse_inline_extra(kind.clone())?;
        let schema = variants
            .get(kind.index().ok_or(object_rainbow::Error::OutOfBounds)?)
            .ok_or(object_rainbow::Error::OutOfBounds)?
            .clone();
        let value = input.parse_extra(schema)?;
        let variants = Extras(variants);
        Ok(Self {
            kind,
            variants,
            value,
        })
    }
}

impl<
    T: AbstractValue + ParseInline<I::WithExtra<Arc<T::Schema>>>,
    I: PointInput<Extra = EnumSchema<T::Schema>>,
> ParseInline<I> for EnumValue<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let EnumSchema { kind, variants } = input.extra().clone();
        let kind: NumericValue = input.parse_inline_extra(kind.clone())?;
        let schema = variants
            .get(kind.index().ok_or(object_rainbow::Error::OutOfBounds)?)
            .ok_or(object_rainbow::Error::OutOfBounds)?
            .clone();
        let value = input.parse_inline_extra(schema)?;
        let variants = Extras(variants);
        Ok(Self {
            kind,
            variants,
            value,
        })
    }
}

impl<T: AbstractValue> AbstractValue for EnumValue<T> {
    type Schema = EnumSchema<T::Schema>;

    fn schema(&self) -> Self::Schema {
        EnumSchema {
            kind: self.kind.schema(),
            variants: self.variants.0.clone(),
        }
    }
}

impl From<EnumValue<InlineValue>> for InlineValue {
    fn from(value: EnumValue<InlineValue>) -> Self {
        Self::Enum(value)
    }
}

impl From<EnumValue<TailValue>> for TailValue {
    fn from(value: EnumValue<TailValue>) -> Self {
        Self::Enum(value)
    }
}
