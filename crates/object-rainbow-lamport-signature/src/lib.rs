use std::ops::Index;

use bitvec::array::BitArray;
use futures_util::future::try_join;
use object_rainbow::{Fetch, FetchBytes, HASH_SIZE, Hash, Singular, SingularFetch, ToOutput, pod};

#[cfg(feature = "generate")]
pub mod generate;

const LENGTH: usize = HASH_SIZE * 8;

#[pod(no_default)]
pub struct FragmentSequence<T>([T; LENGTH]);

impl<T: Default> Default for FragmentSequence<T> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| T::default()))
    }
}

impl<T> FragmentSequence<T> {
    fn map<U>(self, f: impl FnMut(T) -> U) -> FragmentSequence<U> {
        FragmentSequence(self.0.map(f))
    }
}

impl<T> Index<usize> for FragmentSequence<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

#[pod]
pub struct SecretFragment([u8; HASH_SIZE]);

#[pod]
pub struct PrivateFragment(SecretFragment);

#[pod]
pub struct PrivatePair([PrivateFragment; 2]);

#[pod]
pub struct PrivateKey(FragmentSequence<PrivatePair>);

#[pod]
pub struct PublicFragment(Hash);

#[pod]
pub struct PublicPair([PublicFragment; 2]);

#[pod]
pub struct PublicKey(FragmentSequence<PublicPair>);

#[pod]
pub struct SignatureFragment(SecretFragment);

#[pod]
pub struct Signature(FragmentSequence<SignatureFragment>);

#[derive(Clone, Copy)]
pub struct MessageFragment(pub bool);

#[derive(Clone, Copy)]
pub struct Message(pub BitArray<[u8; HASH_SIZE]>);

impl SecretFragment {
    pub fn public(self) -> PublicFragment {
        PublicFragment(self.data_hash())
    }
}

impl PrivateFragment {
    pub fn public(self) -> PublicFragment {
        self.0.public()
    }

    pub fn sign(self) -> SignatureFragment {
        SignatureFragment(self.0)
    }
}

impl PrivatePair {
    pub fn public(self) -> PublicPair {
        PublicPair(self.0.map(PrivateFragment::public))
    }

    pub fn sign(self, message: MessageFragment) -> SignatureFragment {
        self.0[message.0 as usize].sign()
    }
}

impl PrivateKey {
    pub fn public(self) -> PublicKey {
        PublicKey(self.0.map(PrivatePair::public))
    }

    pub fn sign(self, message: Message) -> Signature {
        Signature(FragmentSequence(std::array::from_fn(|n| {
            self.0[n].sign(message.fragment(n))
        })))
    }

    pub fn sign_hash(self, hash: Hash) -> Signature {
        self.sign(hash.into())
    }

    pub fn sign_singular(self, singular: &(impl ?Sized + Singular)) -> Signature {
        self.sign_hash(singular.hash())
    }
}

impl SignatureFragment {
    pub fn verify(self, public: PublicPair, message: MessageFragment) -> bool {
        self.0.public() == public.0[message.0 as usize]
    }
}

impl Signature {
    pub fn verify(self, public: PublicKey, message: Message) -> bool {
        (0..=255).all(|n| self.0[n].verify(public.0[n], message.fragment(n)))
    }
}

impl Message {
    pub fn fragment(&self, n: usize) -> MessageFragment {
        MessageFragment(self.0[n])
    }
}

impl From<Hash> for Message {
    fn from(hash: Hash) -> Self {
        Self(BitArray::new(hash.into()))
    }
}

#[pod]
pub struct Signed<P, S, M> {
    public: P,
    signature: S,
    message: M,
}

impl<P: Fetch<T = PublicKey>, S: Fetch<T = Signature>, M: Singular> Signed<P, S, M> {
    pub async fn sign(private: PrivateKey, public: P, message: M) -> object_rainbow::Result<Self>
    where
        S: From<Signature>,
    {
        if public.fetch().await? != private.public() {
            return Err(object_rainbow::error_operation!("public key mismatch"));
        }
        let signature = private.sign_singular(&message).into();
        Ok(Self {
            public,
            signature,
            message,
        })
    }

    pub async fn new(public: P, signature: S, message: M) -> object_rainbow::Result<Self> {
        let signed = Self {
            public,
            signature,
            message,
        };
        signed.verify().await?;
        Ok(signed)
    }

    pub async fn verify(&self) -> object_rainbow::Result<()> {
        let (public, signature) = try_join(self.public.fetch(), self.signature.fetch()).await?;
        if signature.verify(public, self.message.hash().into()) {
            Ok(())
        } else {
            Err(object_rainbow::error_consistency!("invalid signature"))
        }
    }

    pub async fn message(&self) -> object_rainbow::Result<&M> {
        self.verify().await?;
        Ok(&self.message)
    }
}

impl<P: Fetch<T = PublicKey>, S: Fetch<T = Signature>, M: Singular> FetchBytes for Signed<P, S, M> {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(async move { self.message().await?.fetch_bytes().await })
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(async move { self.message().await?.fetch_data().await })
    }
}

impl<P: Fetch<T = PublicKey>, S: Fetch<T = Signature>, M: Singular> Singular for Signed<P, S, M> {
    fn hash(&self) -> Hash {
        self.message.hash()
    }
}

impl<P: Fetch<T = PublicKey>, S: Fetch<T = Signature>, M: SingularFetch> Fetch for Signed<P, S, M> {
    type T = M::T;

    fn fetch(&'_ self) -> object_rainbow::FailFuture<'_, Self::T> {
        Box::pin(async move { self.message().await?.fetch().await })
    }
}
