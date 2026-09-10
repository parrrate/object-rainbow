use crate::*;

pub struct Local<T> {
    object: T,
}

impl<T: Traversible> Local<T> {
    pub fn new(object: T) -> Self {
        Self { object }
    }
}

impl<T: Traversible + Clone> Fetch for Local<T> {
    type T = T;

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(ready(Ok(self.object.clone())))
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        Ok(Some((self.object.clone(), self.object.to_resolve())))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        Some(self.object.clone())
    }

    fn get(&self) -> Option<&Self::T> {
        Some(&self.object)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(&mut self.object)
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok().map(|Self { object }| object)
    }
}

impl<T: Traversible> FetchBytes for Local<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(Ok((self.object.output(), self.object.to_resolve()))))
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(Ok(self.object.output())))
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        Ok(Some((self.object.output(), self.object.to_resolve())))
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        Some(self.object.output())
    }
}

impl<T: Traversible> Singular for Local<T> {
    fn hash(&self) -> Hash {
        self.object.full_hash()
    }
}
