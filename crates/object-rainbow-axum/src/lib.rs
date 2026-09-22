use std::ops::Deref;

pub struct Refless<T>(pub T);

impl<T> Deref for Refless<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
