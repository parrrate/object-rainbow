use crate::*;

#[derive(Debug, ParseAsInline, Clone, Copy, Default)]
pub struct Nt<T>(pub T);

impl<T> PartialEq for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item: PartialEq>,
{
    fn eq(&self, other: &Self) -> bool {
        self.into_iter().eq(other)
    }
}

impl<T, A: Eq> Eq for Nt<T> where for<'a> &'a T: IntoIterator<Item = A> {}

impl<T, A: PartialOrd> PartialOrd for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.into_iter().partial_cmp(other)
    }
}

impl<T, A: Ord> Ord for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_iter().cmp(other)
    }
}

impl<'a, T> IntoIterator for &'a Nt<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Nt<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for Nt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Nt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ToOutput for Nt<T>
where
    for<'a> &'a T: IntoIterator,
    for<'a> Option<<&'a T as IntoIterator>::Item>: InlineOutput,
{
    fn to_output(&self, output: &mut impl Output) {
        for item in self {
            Some(item).to_output(output);
        }
        None.to_output(output);
    }
}

impl<T> InlineOutput for Nt<T>
where
    for<'a> &'a T: IntoIterator,
    for<'a> Option<<&'a T as IntoIterator>::Item>: InlineOutput,
{
}

impl<T, A: ByteOrd> ByteOrd for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
    Option<A>: ByteOrd + InlineOutput,
{
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.into_iter()
            .map(OrderedByBytes)
            .cmp(other.into_iter().map(OrderedByBytes))
    }
}

impl<T, A: ListHashes> ListHashes for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T, A: Topological> Topological for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: IntoIterator<Item = A> + FromIterator<A>, A, I: ParseInput> ParseInline<I> for Nt<T>
where
    Option<A>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut items = Vec::new();
        while let Some(item) = input.parse_inline()? {
            items.push(item);
        }
        Ok(Self(items.into_iter().collect()))
    }
}
