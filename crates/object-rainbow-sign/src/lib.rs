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
    verify: V,
    signature: S,
    message: M,
}

impl<V: Fetch<T: VerifyKey<S::T>>, S: Fetch, M: Singular> Signed<V, S, M> {
    pub async fn verify(&self) -> object_rainbow::Result<()> {
        let (verifykey, signature) = (self.verify.fetch(), self.signature.fetch())
            .try_join()
            .await?;
        verifykey
            .verify(&signature, &self.message.hash())
            .map_err(object_rainbow::Error::consistency)
    }
}
