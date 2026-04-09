mod private;

pub type Le<T> = <T as private::AsLe>::Le;
pub type Be<T> = <T as private::AsBe>::Be;

/// Construct a little-endian value.
#[allow(non_snake_case)]
pub fn Le<T: private::AsLe>(n: T) -> T::Le {
    n.construct()
}
/// Construct a big-endian value.
#[allow(non_snake_case)]
pub fn Be<T: private::AsBe>(n: T) -> T::Be {
    n.construct()
}
