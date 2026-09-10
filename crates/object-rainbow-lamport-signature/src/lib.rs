use std::ops::Index;

use bitvec::array::BitArray;
use object_rainbow::{
    HASH_SIZE, Hash, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged,
    ToOutput, Topological, pod,
};

const LENGTH: usize = HASH_SIZE * 8;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
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

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct SignatureFragment(SecretFragment);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
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
}

impl SignatureFragment {
    pub fn validate(self, public: PublicPair, message: MessageFragment) -> bool {
        self.0.public() == public.0[message.0 as usize]
    }
}

impl Signature {
    pub fn validate(self, public: PublicKey, message: Message) -> bool {
        (0..=255).all(|n| self.0[n].validate(public.0[n], message.fragment(n)))
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
