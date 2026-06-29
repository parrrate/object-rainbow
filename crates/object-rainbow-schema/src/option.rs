use crate::*;

#[derive(Debug, ListHashes, Topological, Tagged, PartialEq)]
#[rainbow(untagged)]
pub enum OptionValue<T: AbstractValue> {
    None(Arc<T::Schema>),
    Some(Shared<T>),
}

impl<T: AbstractValue> ToOutput for OptionValue<T> {
    fn to_output(&self, output: &mut impl Output) {
        match self {
            Self::None(schema) => schema.none_output(output),
            Self::Some(value) => value.some_output(output),
        }
    }
}

impl<T: AbstractValue + InlineOutput> InlineOutput for OptionValue<T> {}

impl<T: AbstractValue> OptionValue<T> {
    pub fn inner_schema(&self) -> Arc<T::Schema> {
        match self {
            Self::None(schema) => schema.clone(),
            Self::Some(value) => Arc::new(value.schema()),
        }
    }
}

impl<T: AbstractValue + Parse<I>, I: PointInput<Extra = Arc<T::Schema>>> Parse<I>
    for OptionValue<T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        let niche = schema.niche();
        if niche.needs_tag() {
            match input.parse_inline::<u8>()? {
                0xff => Ok(Self::Some(input.parse()?)),
                0xfe => Ok(Self::None(schema)),
                _ => Err(object_rainbow::Error::OutOfBounds),
            }
        } else {
            input.parse_compare(&niche.vec()).map(|value| match value {
                Some(value) => Self::Some(value),
                None => Self::None(schema),
            })
        }
    }
}

impl<T: AbstractValue + ParseInline<I>, I: PointInput<Extra = Arc<T::Schema>>> ParseInline<I>
    for OptionValue<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        let niche = schema.niche();
        if niche.needs_tag() {
            match input.parse_inline::<u8>()? {
                0xff => Ok(Self::Some(input.parse_inline()?)),
                0xfe => Ok(Self::None(schema)),
                _ => Err(object_rainbow::Error::OutOfBounds),
            }
        } else {
            input
                .parse_compare_inline(&niche.vec())
                .map(|value| match value {
                    Some(value) => Self::Some(value),
                    None => Self::None(schema),
                })
        }
    }
}

impl<T: AbstractValue<Schema: OptionSchema>> AbstractValue for OptionValue<T> {
    type Schema = T::Schema;

    fn schema(&self) -> Self::Schema {
        self.inner_schema().option()
    }
}
