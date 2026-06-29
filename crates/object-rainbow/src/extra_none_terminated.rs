use crate::{extras::Extras, none_terminated::Nt, *};

#[derive(Debug, ListHashes, Topological, Parse, ParseInline, Tagged)]
pub struct Ent<T, E = ()> {
    pub extra: Extras<E>,
    pub items: Nt<T>,
}

impl<T: IntoIterator> IntoIterator for Ent<T> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
