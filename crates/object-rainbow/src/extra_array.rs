use crate::{
    extras::Extras, map_extra::MappedExtra, runtime_array::RuntimeArray, tuple_extra::Extra1, *,
};

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse)]
pub struct ExtraArray<T, E> {
    pub extra: MappedExtra<Extras<E>, Extra1>,
    pub items: RuntimeArray<T>,
}

impl<T, E: Clone> CanonicalExtra for ExtraArray<T, E> {
    type Extra = (u64, E);

    fn canonical_extra(&self) -> Self::Extra {
        (self.items.len() as _, self.extra.1.0.clone())
    }
}
