use crate::*;

pub struct FnFetch<F> {
    fetch: Arc<F>,
}

impl<F> Clone for FnFetch<F> {
    fn clone(&self) -> Self {
        Self {
            fetch: self.fetch.clone(),
        }
    }
}

pub trait FetchFn: Send + Sync {
    type T;
    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>>;
}

impl<F: Send + Sync + Fn() -> Fut, Fut: Send + Future<Output = Result<T>>, T> FetchFn for F {
    type T = T;

    fn fetch(&self) -> impl Send + Future<Output = Result<Self::T>> {
        self()
    }
}

impl<F: FetchFn<T: Clone>> FnFetch<F> {
    pub fn new(fetch: F) -> Self {
        let fetch = fetch.into();
        Self { fetch }
    }

    pub async fn fetch(&self) -> Result<F::T> {
        self.fetch.fetch().await
    }

    async fn fetch_node(&self) -> Result<Node<F::T>>
    where
        F::T: Traversible,
    {
        let object = self.fetch().await?;
        let resolve = object.clone().into_resolve();
        Ok((object, resolve))
    }
}

impl<F: FetchFn<T: Clone + Traversible>> FetchBytes for FnFetch<F> {
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let (object, resolve) = self.fetch_node().await?;
            let data = object.output();
            Ok((data, resolve))
        })
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::pin(async move { Ok(self.fetch().await?.output()) })
    }
}

impl<F: FetchFn<T: Clone + Traversible>> Fetch for FnFetch<F> {
    type T = F::T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        Box::pin(async move { self.fetch().await })
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
    }
}

pub trait ClosureFn<'a, Closure: 'a>: Send + Sync + Fn(&'a Closure) -> Self::Fut {
    type T;
    type Fut: Send + Future<Output = Result<Self::T>>;
    fn fetch(&'a self, closure: &'a Closure) -> Self::Fut;
}

impl<
    'a,
    Closure: 'a,
    F: Send + Sync + Fn(&'a Closure) -> Fut,
    Fut: Send + Future<Output = Result<T>>,
    T,
> ClosureFn<'a, Closure> for F
{
    type T = T;
    type Fut = Fut;

    fn fetch(&'a self, closure: &'a Closure) -> Self::Fut {
        self(closure)
    }
}

pub trait ClosureFetch<Closure>: Send + Sync {
    type T;
    fn fetch(&self, closure: &Closure) -> impl Send + Future<Output = Result<Self::T>>;
}

impl<Closure: Send + Sync, F: for<'a> ClosureFn<'a, Closure, T = T>, T> ClosureFetch<Closure>
    for F
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
) -> (Closure, F) {
    (closure, f)
}
