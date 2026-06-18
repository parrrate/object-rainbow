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

impl<T: FromIterator<A>, M: Monostate, A> FromIterator<A> for MonostateHeaders<T, M> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
            monostate: Default::default(),
        }
    }
}

pub struct Iter<I, M> {
    inner: I,
    monostate: M,
}

impl<I: Iterator, M: Monostate> Iterator for Iter<I, M> {
    type Item = (M, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.monostate, item))
    }
}

impl<'a, T, M: Monostate> IntoIterator for &'a MonostateHeaders<T, M>
where
    &'a T: IntoIterator,
{
    type Item = (M, <&'a T as IntoIterator>::Item);

    type IntoIter = Iter<<&'a T as IntoIterator>::IntoIter, M>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.inner.into_iter(),
            monostate: self.monostate,
        }
    }
}
