use crate::*;

pub struct FnFetch<F> {
    fetch: F,
}

pub trait FetchFn: Send + Sync {
    type T: Traversible;
    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>>;
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

    async fn fetch_node(&self) -> Result<Node<T>> {
        let object = self.fetch().await?;
        let resolve = object.to_resolve();
        Ok((object, resolve))
    }
}

impl<F: Send + Sync + Fn() -> Fut, Fut: Send + Future<Output = Result<T>>, T: Traversible>
    FetchBytes for FnFetch<F>
{
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(async move {
            let (object, resolve) = self.fetch_node().await?;
            let data = object.output();
            Ok((data, resolve))
        })
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move { Ok(self.fetch().await?.output()) })
    }
}

impl<F: Send + Sync + Fn() -> Fut, Fut: Send + Future<Output = Result<T>>, T: Traversible> Fetch
    for FnFetch<F>
{
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move { self.fetch_node().await })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move { self.fetch().await })
    }
}
