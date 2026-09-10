use bitvec::array::BitArray;
use object_rainbow::{HASH_SIZE, Hash, InlineOutput, ToOutput};

const LENGTH: usize = HASH_SIZE * 8;

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct SecretFragment([u8; HASH_SIZE]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivateFragment(SecretFragment);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivatePair([PrivateFragment; 2]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivateKey([PrivatePair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy, PartialEq, Eq)]
pub struct PublicFragment(Hash);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy, PartialEq, Eq)]
pub struct PublicPair([PublicFragment; 2]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey([PublicPair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct SignatureFragment(SecretFragment);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct Signature([SignatureFragment; LENGTH]);

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

    pub fn sign(self, data: MessageFragment) -> SignatureFragment {
        self.0[data.0 as usize].sign()
    }
}

impl PrivateKey {
    pub fn public(self) -> PublicKey {
        PublicKey(self.0.map(PrivatePair::public))
    }

    pub fn sign(self, data: Message) -> Signature {
        Signature(std::array::from_fn(|n| self.0[n].sign(data.fragment(n))))
    }
}

impl SignatureFragment {
    pub fn validate(self, public: PublicPair, data: MessageFragment) -> bool {
        self.0.public() == public.0[data.0 as usize]
    }
}

impl Signature {
    pub fn validate(self, public: PublicKey, data: Message) -> bool {
        (0..=255).all(|n| self.0[n].validate(public.0[n], data.fragment(n)))
    }
}

impl Message {
    pub fn fragment(&self, n: usize) -> MessageFragment {
        MessageFragment(self.0[n])
    }
}
