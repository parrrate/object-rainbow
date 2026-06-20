use std::sync::Arc;

use crate::*;

#[derive(ParseAsInline)]
pub struct Dt<T, A> {
    inner: Arc<(T, A)>,
}

impl<T, A> Clone for Dt<T, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T, A> Deref for Dt<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl<T, A: Default + InlineOutput> ToOutput for Dt<T, A>
where
    for<'a> &'a T: IntoIterator<Item: InlineOutput>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
        self.inner.1.to_output(output);
    }
}

impl<T, A: Default + InlineOutput> InlineOutput for Dt<T, A> where
    for<'a> &'a T: IntoIterator<Item: InlineOutput>
{
}

impl<T, A> ListHashes for Dt<T, A>
where
    for<'a> &'a T: IntoIterator<Item: ListHashes>,
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T, A> Topological for Dt<T, A>
where
    for<'a> &'a T: IntoIterator<Item: Topological>,
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: Tagged, A> Tagged for Dt<T, A> {
    const TAGS: Tags = T::TAGS;
}

impl<T: FromIterator<A>, A: PartialEq + Default + ParseInline<I>, I: ParseInput> ParseInline<I>
    for Dt<T, A>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut items = Vec::new();
        let default = A::default();
        let term = loop {
            let item = input.parse_inline()?;
            if item == default {
                break item;
            }
            items.push(item);
        };
        let inner = Arc::new((items.into_iter().collect(), term));
        Ok(Self { inner })
    }
}
