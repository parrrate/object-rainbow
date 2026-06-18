use crate::*;

#[derive(Debug, ParseAsInline, Clone, Copy, Default)]
pub struct Sequence<T>(pub T);

impl<T> Deref for Sequence<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
