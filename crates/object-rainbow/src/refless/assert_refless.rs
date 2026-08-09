use crate::ListHashes;

pub struct AssertRefless<T>(pub T);

impl<T> ListHashes for AssertRefless<T> {}
