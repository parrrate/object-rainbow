use object_rainbow::extra_option::ExtraOption;

use crate::*;

pub type OptionValue<T> = ExtraOption<Shared<T>, Arc<<T as AbstractValue>::Schema>>;

impl<T: AbstractValue<Schema: OptionSchema>> AbstractValue for OptionValue<T> {
    type Schema = T::Schema;

    fn schema(&self) -> Self::Schema {
        self.canonical_extra().option()
    }
}

impl<T: AbstractValue + AbstractCollection> AbstractCollection for OptionValue<T> {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        match self {
            Self::None(_) => Vec::new(),
            Self::Some(value) => value.items(),
        }
    }
}
