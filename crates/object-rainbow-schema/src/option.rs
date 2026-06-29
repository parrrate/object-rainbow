use crate::*;

#[derive(Debug, ListHashes, Topological, Tagged, PartialEq)]
#[rainbow(untagged)]
pub enum OptionValue<T: AbstractValue> {
    None(Arc<T::Schema>),
    Some(Shared<T>),
}
