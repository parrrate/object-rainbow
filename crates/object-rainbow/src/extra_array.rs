use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, PartialEq)]
pub struct ExtraArray<T>(pub Vec<T>);

/// Here we assume extra will match on re-parsing, which is a general requirement for
/// correctness anyway.
impl<T: InlineOutput> InlineOutput for ExtraArray<T> {}
