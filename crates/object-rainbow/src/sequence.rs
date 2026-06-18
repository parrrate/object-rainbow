use crate::*;

#[derive(Debug, ParseAsInline, Clone, Copy, Default)]
pub struct Sequence<T>(pub T);

impl<T> Deref for Sequence<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
