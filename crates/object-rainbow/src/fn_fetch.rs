use crate::*;

pub struct FnFetch<F> {
    pub fetch: F,
}

impl<F: Fn() -> Fut, Fut: Future<Output: ToOutput + Traversible>> FnFetch<F> {
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }
}
