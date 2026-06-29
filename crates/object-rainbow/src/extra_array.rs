use crate::*;

#[derive(ToOutput, InlineOutput)]
pub struct ExtraArray<T>(pub Vec<T>);
