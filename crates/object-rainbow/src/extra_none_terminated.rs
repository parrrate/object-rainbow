use crate::{extras::Extras, none_terminated::Nt, *};

#[derive(Debug, ListHashes, Topological, Parse, ParseInline, Tagged)]
pub struct Ent<T, E = ()> {
    pub extra: Extras<E>,
    pub items: Nt<T>,
}

impl<T, E: PartialEq> PartialEq for Ent<T, E>
where
    for<'a> &'a T: IntoIterator<Item: PartialEq>,
{
    fn eq(&self, other: &Self) -> bool {
        self.extra == other.extra && self.items == other.items
    }
}

impl<T: IntoIterator> IntoIterator for Ent<T> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Ent<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Ent<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
