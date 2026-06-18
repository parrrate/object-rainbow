use crate::*;

#[derive(Debug, ParseAsInline, Clone, Copy, Default)]
pub struct Sequence<T>(pub T);

impl<T> PartialEq for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: PartialEq>,
{
    fn eq(&self, other: &Self) -> bool {
        self.into_iter().eq(other)
    }
}

impl<T> Eq for Sequence<T> where for<'a> &'a T: IntoIterator<Item: Eq> {}

impl<T> Deref for Sequence<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> IntoIterator for &'a Sequence<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Sequence<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
