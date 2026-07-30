use crate::FetchBytes;

pub struct DelayedExtra<E, F, T>(pub E, pub F, pub T);

impl<E, F: FetchBytes, T> FetchBytes for DelayedExtra<E, F, T> {
    fn fetch_bytes(&'_ self) -> crate::FailFuture<'_, crate::ByteNode> {
        self.1.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> crate::FailFuture<'_, Vec<u8>> {
        self.1.fetch_data()
    }
}
