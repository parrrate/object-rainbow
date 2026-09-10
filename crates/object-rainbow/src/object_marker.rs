use std::fmt::Debug;

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

impl<T: ?Sized> PartialEq for ObjectMarker<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl<T: ?Sized> Eq for ObjectMarker<T> {}

impl<T: ?Sized> PartialOrd for ObjectMarker<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ?Sized> Ord for ObjectMarker<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.object.cmp(&other.object)
    }
}

impl<T: ?Sized> core::hash::Hash for ObjectMarker<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.object.hash(state);
    }
}

impl<T: ?Sized> Debug for ObjectMarker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectMarker")
            .field("object", &self.object)
            .finish()
    }
}
