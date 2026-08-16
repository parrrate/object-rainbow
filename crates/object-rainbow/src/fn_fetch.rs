use crate::*;

pub struct FnFetch<F> {
    pub fetch: F,
}

impl<F: Fn() -> Fut, Fut: Future<Output: Traversible>> FnFetch<F> {
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }
}
