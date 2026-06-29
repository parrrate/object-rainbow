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
