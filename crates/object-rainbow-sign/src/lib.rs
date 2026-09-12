use futures_concurrency::future::TryJoin;
use object_rainbow::{Fetch, Hash, Singular, pod};

pub trait SigningKey<Message = Hash> {
    type Signature;
    type VerifyKey: VerifyKey<Self::Signature, Message>;
    fn sign(&self, message: &Message) -> Self::Signature;
    fn to_verify_key(&self) -> Self::VerifyKey;
}

pub trait VerifyKey<Signature, Message = Hash> {
    type Error: 'static + Send + Sync + std::error::Error;
    fn verify(&self, signature: &Signature, message: &Message) -> Result<(), Self::Error>;
}

#[pod]
pub struct Signed<V, S, M> {
    verify_key: V,
    signature: S,
    message: M,
}

impl<V: Fetch<T: VerifyKey<S::T>>, S: Fetch, M: Singular> Signed<V, S, M> {
    pub async fn verify(&self) -> object_rainbow::Result<()> {
        let (verify_key, signature) = (self.verify_key.fetch(), self.signature.fetch())
            .try_join()
            .await?;
        verify_key
            .verify(&signature, &self.message.hash())
            .map_err(object_rainbow::Error::consistency)
    }

    pub async fn new(verify_key: V, signature: S, message: M) -> object_rainbow::Result<Self> {
        let signed = Self {
            verify_key,
            signature,
            message,
        };
        signed.verify().await?;
        Ok(signed)
    }

    pub async fn message(&self) -> object_rainbow::Result<&M> {
        self.verify().await?;
        Ok(&self.message)
    }
}
