use crate::*;

#[derive(ToOutput, InlineOutput, ListHashes)]
pub struct ExtraArray<T>(pub Vec<T>);
