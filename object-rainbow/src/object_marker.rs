use crate::*;

#[derive(ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline)]
pub struct ObjectMarker<T: ?Sized> {
    object: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Clone for ObjectMarker<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for ObjectMarker<T> {}

impl<T: ?Sized> Default for ObjectMarker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> ObjectMarker<T> {
    pub const fn new() -> Self {
        Self {
            object: PhantomData,
        }
    }
}

impl<T: ?Sized + Tagged> Tagged for ObjectMarker<T> {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}
