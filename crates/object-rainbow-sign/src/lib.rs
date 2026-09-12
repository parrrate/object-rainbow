use futures_concurrency::future::TryJoin;
use object_rainbow::{Fetch, FetchBytes, FullHash, Hash, Singular, SingularFetch, pod};

#[cfg(feature = "signature")]
mod signature;

pub trait SigningKey<V: VerifyKey<S, M>, S, M = Hash> {
    fn sign(&self, message: &M) -> S;
    fn to_verify_key(&self) -> V;
}

pub trait VerifyKey<S, M = Hash>: Send + Sync + Eq {
    type Error: 'static + Send + Sync + std::error::Error;
    fn verify(&self, signature: &S, message: &M) -> Result<(), Self::Error>;
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

    fn sign_trusted<K: SigningKey<V::T, S::T>>(
        signing_key: &K,
        verify_key: V,
        message: M,
    ) -> object_rainbow::Result<Self>
    where
        S: From<S::T>,
    {
        let signature = signing_key.sign(&message.hash()).into();
        Ok(Self {
            verify_key,
            signature,
            message,
        })
    }

    pub async fn sign_fetch<K: SigningKey<V::T, S::T>>(
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
        Self::sign_trusted(signing_key, verify_key, message)
    }

    pub fn sign<K: SigningKey<V::T, S::T>>(
        signing_key: &K,
        verify_key: V,
        message: M,
    ) -> object_rainbow::Result<Self>
    where
        S: From<S::T>,
        V: SingularFetch<T: FullHash>,
    {
        if verify_key.hash() != signing_key.to_verify_key().full_hash() {
            return Err(object_rainbow::error_operation!("public key mismatch"));
        }
        Self::sign_trusted(signing_key, verify_key, message)
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

impl<V: Fetch<T: VerifyKey<S::T>>, S: Fetch<T: Send + Sync>, M: Singular> Singular
    for Signed<V, S, M>
{
    fn hash(&self) -> Hash {
        self.message.hash()
    }
}

impl<V: Fetch<T: VerifyKey<S::T>>, S: Fetch<T: Send + Sync>, M: SingularFetch> Fetch
    for Signed<V, S, M>
{
    type T = M::T;

    fn fetch(&'_ self) -> object_rainbow::FailFuture<'_, Self::T> {
        Box::pin(async move { self.message().await?.fetch().await })
    }
}
