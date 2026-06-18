use crate::*;

#[derive(Debug, ParseAsInline, Clone, Copy, Default)]
pub struct Sequence<T>(pub T);
