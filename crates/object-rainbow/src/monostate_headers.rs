use crate::*;

#[derive(Default)]
pub struct MonostateHeaders<T, M = ()> {
    pub inner: T,
    monostate: M,
}

impl<T, M> Deref for MonostateHeaders<T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, M> DerefMut for MonostateHeaders<T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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

impl<T: IntoIterator, M: Monostate> IntoIterator for MonostateHeaders<T, M> {
    type Item = (M, T::Item);
    type IntoIter = Iter<T::IntoIter, M>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
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

impl<'a, T, M: Monostate> IntoIterator for &'a mut MonostateHeaders<T, M>
where
    &'a mut T: IntoIterator,
{
    type Item = (M, <&'a mut T as IntoIterator>::Item);

    type IntoIter = Iter<<&'a mut T as IntoIterator>::IntoIter, M>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.inner.into_iter(),
            monostate: self.monostate,
        }
    }
}
