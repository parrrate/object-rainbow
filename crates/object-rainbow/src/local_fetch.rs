use crate::*;

pub struct Local<T>(pub T);

impl<T> Deref for Local<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Traversible + Clone> Fetch for Local<T> {
    type T = T;

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(ready(Ok(self.0.clone())))
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        Ok(Some((self.0.clone(), self.0.to_resolve())))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        Some(self.0.clone())
    }

    fn get(&self) -> Option<&Self::T> {
        Some(&self.0)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(&mut self.0)
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok().map(|Self(object)| object)
    }
}

impl<T: Traversible> FetchBytes for Local<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(Ok((self.0.output(), self.0.to_resolve()))))
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(Ok(self.0.output())))
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        Ok(Some((self.0.output(), self.0.to_resolve())))
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        Some(self.0.output())
    }
}

impl<T: Traversible> Singular for Local<T> {
    fn hash(&self) -> Hash {
        self.0.full_hash()
    }
}
