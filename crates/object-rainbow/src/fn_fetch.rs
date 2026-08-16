use crate::*;

pub struct FnFetch<F> {
    fetch: F,
}

impl<F: Fn() -> Fut, Fut: Future<Output = Result<T>>, T: Traversible> FnFetch<F> {
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }

    pub async fn fetch(&self) -> Result<T> {
        (self.fetch)().await
    }
}
