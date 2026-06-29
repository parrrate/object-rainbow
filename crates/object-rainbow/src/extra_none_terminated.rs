use crate::{extra_none::ExtraNoneOutput, extras::Extras, none_terminated::Nt, *};

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

impl<T: IntoIterator, E> IntoIterator for Ent<T, E> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T, E> IntoIterator for &'a Ent<T, E>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T, E> IntoIterator for &'a mut Ent<T, E>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub trait EntOutput<E> {
    fn ent_output(self, extra: &E, output: &mut impl Output);
}

impl<T: IntoIterator<Item = A>, E, A: ExtraNoneOutput<E> + InlineOutput> EntOutput<E> for T {
    fn ent_output(self, extra: &E, output: &mut impl Output) {
        for item in self {
            item.extra_some_output(output);
        }
        A::extra_none_output(extra, output);
    }
}

impl<T, E> ToOutput for Ent<T, E>
where
    for<'a> &'a T: EntOutput<E>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.items.ent_output(&self.extra, output);
    }
}

impl<T, E> InlineOutput for Ent<T, E> where Self: ToOutput {}
