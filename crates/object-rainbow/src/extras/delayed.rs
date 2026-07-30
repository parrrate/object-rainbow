use crate::FetchBytes;

pub struct DelayedExtra<E, F>(pub E, pub F);

impl<E, F: FetchBytes> FetchBytes for DelayedExtra<E, F> {
    fn fetch_bytes(&'_ self) -> crate::FailFuture<'_, crate::ByteNode> {
        self.1.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> crate::FailFuture<'_, Vec<u8>> {
        self.1.fetch_data()
    }
}
