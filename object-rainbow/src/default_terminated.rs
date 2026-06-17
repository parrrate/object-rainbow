use std::sync::Arc;

use crate::*;

#[derive(ParseAsInline)]
pub struct Dt<T> {
    inner: Arc<T>,
}

impl<T> Clone for Dt<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for Dt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, A: Default + InlineOutput> ToOutput for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
        A::default().to_output(output);
    }
}

impl<T, A: Default + InlineOutput> InlineOutput for Dt<T> where
    for<'a> &'a T: IntoIterator<Item = &'a A>
{
}

impl<T, A: ListHashes> ListHashes for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T, A: Topological> Topological for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: FromIterator<A>, A: PartialEq + Default + ParseInline<I>, I: ParseInput> ParseInline<I>
    for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut items = Vec::new();
        let default = A::default();
        while let item = input.parse_inline()?
            && item != default
        {
            items.push(item);
        }
        let inner = Arc::new(items.into_iter().collect());
        Ok(Self { inner })
    }
}
