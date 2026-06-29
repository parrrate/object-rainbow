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
