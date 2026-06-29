use crate::{
    extras::Extras, map_extra::MappedExtra, runtime_array::RuntimeArray, tuple_extra::Extra1,
};

pub struct ExtraArray<T, E> {
    pub extra: MappedExtra<Extras<E>, Extra1>,
    pub items: RuntimeArray<T>,
}
