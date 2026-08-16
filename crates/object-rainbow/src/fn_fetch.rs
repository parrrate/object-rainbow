use crate::*;

pub struct FnFetch<F> {
    fetch: F,
}

impl<F: Send + Sync + Fn() -> Fut, Fut: Send + Future<Output = Result<T>>, T: Traversible>
    FnFetch<F>
{
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }

    pub async fn fetch(&self) -> Result<T> {
        (self.fetch)().await
    }

    pub async fn fetch_node(&self) -> Result<Node<T>> {
        let object = self.fetch().await?;
        let resolve = object.to_resolve();
        Ok((object, resolve))
    }
}
