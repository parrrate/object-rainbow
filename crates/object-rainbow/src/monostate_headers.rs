use crate::*;

pub struct MonostateHeaders<T, M = ()> {
    pub inner: T,
    monostate: M,
}

pub struct IntoIter<I, M> {
    inner: I,
    monostate: M,
}

impl<I: Iterator, M: Monostate> Iterator for IntoIter<I, M> {
    type Item = (M, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.monostate, item))
    }
}

impl<T: IntoIterator, M: Monostate> IntoIterator for MonostateHeaders<T, M> {
    type Item = (M, T::Item);
    type IntoIter = IntoIter<T::IntoIter, M>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
            monostate: self.monostate,
        }
    }
}
