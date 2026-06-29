use crate::*;

#[derive(ToOutput, InlineOutput, ListHashes, Topological)]
pub struct ExtraArray<T>(pub Vec<T>);
