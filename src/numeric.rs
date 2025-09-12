mod private;

pub type Le<T> = <T as private::AsLe>::Le;
pub type Be<T> = <T as private::AsBe>::Be;
