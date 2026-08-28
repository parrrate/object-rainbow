use crate::Key;

#[derive(Debug, thiserror::Error)]
pub enum LayeredError<O, I> {
    #[error(transparent)]
    Outer(O),
    #[error(transparent)]
    Inner(I),
}

impl<O: Key, I: Key> Key for (O, I) {
    type Error = LayeredError<O::Error, I::Error>;

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        self.0.encrypt(&self.1.encrypt(data))
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
        self.1
            .decrypt(&self.0.decrypt(data).map_err(LayeredError::Outer)?)
            .map_err(LayeredError::Inner)
    }
}
