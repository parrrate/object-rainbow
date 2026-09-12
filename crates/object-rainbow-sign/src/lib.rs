use futures_concurrency::future::TryJoin;
use object_rainbow::{Fetch, FetchBytes, Hash, Singular, pod};

pub trait SigningKey<Message = Hash> {
    type Signature: Send + Sync;
    type VerifyKey: VerifyKey<Self::Signature, Message>;
    fn sign(&self, message: &Message) -> Self::Signature;
    fn to_verify_key(&self) -> Self::VerifyKey;
}

pub trait VerifyKey<Signature, Message = Hash>: Send + Sync + Eq {
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

    pub async fn sign<K: SigningKey<Signature = S::T, VerifyKey = V::T>>(
        signing_key: &K,
        verify_key: V,
        message: M,
    ) -> object_rainbow::Result<Self>
    where
        S: From<S::T>,
    {
        if verify_key.fetch().await? != signing_key.to_verify_key() {
            return Err(object_rainbow::error_operation!("public key mismatch"));
        }
        let signature = signing_key.sign(&message.hash()).into();
        Ok(Self {
            verify_key,
            signature,
            message,
        })
    }
}

impl<V: Fetch<T: VerifyKey<S::T>>, S: Fetch<T: Send + Sync>, M: Singular> FetchBytes
    for Signed<V, S, M>
{
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(async move { self.message().await?.fetch_bytes().await })
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(async move { self.message().await?.fetch_data().await })
    }
}
