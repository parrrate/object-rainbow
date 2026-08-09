use crate::{ListHashes, Topological};

pub struct AssertRefless<T>(pub T);

impl<T> ListHashes for AssertRefless<T> {}
impl<T> Topological for AssertRefless<T> {}
