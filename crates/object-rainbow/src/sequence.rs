use crate::*;

#[derive(Debug, Clone, Copy, Default)]
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

impl<T> PartialOrd for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: PartialOrd>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.into_iter().partial_cmp(other)
    }
}

impl<T> Ord for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: Ord>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_iter().cmp(other)
    }
}

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

impl<T: FromIterator<A>, A> FromIterator<A> for Sequence<T> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: IntoIterator> IntoIterator for Sequence<T> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

impl<T> ToOutput for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: InlineOutput>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
    }
}

impl<T> ByteOrd for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: ByteOrd + InlineOutput>,
{
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.iter_bytes_cmp(other)
    }
}

impl<T> ListHashes for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: ListHashes>,
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T> Topological for Sequence<T>
where
    for<'a> &'a T: IntoIterator<Item: Topological>,
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: IntoIterator<Item = A> + FromIterator<A>, A: ParseInline<I>, I: ParseInput> Parse<I>
    for Sequence<T>
{
    fn parse(input: I) -> crate::Result<Self> {
        Ok(Self(input.parse_collect()?))
    }
}

pub trait PlainCollection: IntoIterator {}

pub trait VecLike: IntoIterator {}

impl<T: IntoIterator> VecLike for Sequence<T> {}
