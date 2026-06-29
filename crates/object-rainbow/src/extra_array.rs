use crate::*;

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, PartialEq)]
pub struct ExtraArray<T>(pub Vec<T>);
