use crate::*;

#[derive(Clone)]
pub struct LocalFetch<T> {
    object: T,
}

impl<T: Traversible> LocalFetch<T> {
    pub fn new(object: T) -> Self {
        Self { object }
    }
}

impl<T: Traversible> Fetch for LocalFetch<T> {
    type T = T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        Box::pin(ready(Ok(self.object)))
    }

    fn get(&self) -> Option<&Self::T> {
        Some(&self.object)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(&mut self.object)
    }

    fn try_unwrap(self: Box<Self>) -> Option<Self::T> {
        Some(self.object)
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
    }
}

impl<T: Traversible> FetchBytes for LocalFetch<T> {
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::pin(ready(Ok((
            self.object.output(),
            self.object.into_resolve(),
        ))))
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::pin(ready(Ok(self.object.output())))
    }
}

impl<T: Traversible> Singular for LocalFetch<T> {
    fn hash(&self) -> Hash {
        self.object.full_hash()
    }
}
