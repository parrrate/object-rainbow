use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, PartialEq, ParseAsInline)]
pub struct ExtraArray<T>(pub Vec<T>);

impl<A> FromIterator<A> for ExtraArray<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Here we assume extra will match on re-parsing, which is a general requirement for
/// correctness anyway.
impl<T: InlineOutput> InlineOutput for ExtraArray<T> {}
