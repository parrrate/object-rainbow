use crate::*;

pub struct FnFetch<F> {
    fetch: F,
}

pub trait FetchFn: Send + Sync {
    type T: Traversible;
    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>>;
}

impl<F: Send + Sync + Fn() -> Fut, Fut: Send + Future<Output = Result<T>>, T: Traversible> FetchFn
    for F
{
    type T = T;

    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>> {
        self()
    }
}

impl<F: FetchFn> FnFetch<F> {
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }

    pub async fn fetch(&self) -> Result<F::T> {
        self.fetch.fetch().await
    }

    async fn fetch_node(&self) -> Result<Node<F::T>> {
        let object = self.fetch().await?;
        let resolve = object.to_resolve();
        Ok((object, resolve))
    }
}

impl<F: FetchFn> FetchBytes for FnFetch<F> {
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

impl<F: FetchFn> Fetch for FnFetch<F> {
    type T = F::T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move { self.fetch_node().await })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move { self.fetch().await })
    }
}

pub trait ClosureFn<'a, Closure: 'a>: Send + Sync + Fn(&'a Closure) -> Self::Fut {
    type T: Traversible;
    type Fut: Send + Future<Output = Result<Self::T>>;
    fn fetch(&'a self, closure: &'a Closure) -> Self::Fut;
}

impl<
    'a,
    Closure: 'a,
    F: Send + Sync + Fn(&'a Closure) -> Fut,
    Fut: Send + Future<Output = Result<T>>,
    T: Traversible,
> ClosureFn<'a, Closure> for F
{
    type T = T;
    type Fut = Fut;

    fn fetch(&'a self, closure: &'a Closure) -> Self::Fut {
        self(closure)
    }
}

pub trait ClosureFetch<Closure>: Send + Sync {
    type T: Traversible;
    fn fetch(&self, closure: &Closure) -> impl Send + Future<Output = Result<Self::T>>;
}

impl<Closure: Send + Sync, F: for<'a> ClosureFn<'a, Closure, T = T>, T: Traversible>
    ClosureFetch<Closure> for F
{
    type T = T;

    async fn fetch(&self, closure: &Closure) -> Result<Self::T> {
        self.fetch(closure).await
    }
}

impl<Closure: Send + Sync, F: ClosureFetch<Closure>> FetchFn for (Closure, F) {
    type T = F::T;

    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>> {
        self.1.fetch(&self.0)
    }
}

pub fn closure_fetch<
    Closure: Send + Sync,
    F: ClosureFetch<Closure> + AsyncFn(&Closure) -> Result<F::T>,
>(
    closure: Closure,
    f: F,
) -> impl FetchFn<T = F::T> {
    (closure, f)
}
