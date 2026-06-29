use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, PartialEq, ParseAsInline)]
pub struct RuntimeArray<T>(pub Vec<T>);

impl<T> Deref for RuntimeArray<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RuntimeArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<A> FromIterator<A> for RuntimeArray<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Here we assume extra will match on re-parsing, which is a general requirement for correctness
/// anyway.
impl<T: InlineOutput> InlineOutput for RuntimeArray<T> {}

impl<T: ParseInline<I::WithExtra<E>>, I: PointInput<Extra = (u64, E)>, E: 'static + Clone>
    ParseInline<I> for RuntimeArray<T>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let (n, extra) = input.extra().clone();
        (0..n)
            .map(|_| input.parse_inline_extra(extra.clone()))
            .collect()
    }
}
