use object_rainbow::{HASH_SIZE, Hash, InlineOutput, ToOutput};

const LENGTH: usize = HASH_SIZE * 8;

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivateFragment([u8; HASH_SIZE]);

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivatePair([PrivateFragment; 2]);

#[derive(Debug, ToOutput, Clone, Copy)]
pub struct PrivateKey([PrivatePair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PublicFragment(Hash);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PublicPair([PublicFragment; 2]);

#[derive(Debug, ToOutput)]
pub struct PublicKey([PublicPair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct SignatureFragment(Hash);

#[derive(Debug, ToOutput)]
pub struct Signature([Hash; LENGTH]);

pub struct MessageFragment(pub bool);

impl PrivateFragment {
    pub fn public(self) -> PublicFragment {
        PublicFragment(self.data_hash())
    }
}

impl PrivatePair {
    pub fn public(self) -> PublicPair {
        PublicPair(self.0.map(PrivateFragment::public))
    }
}

impl PrivateKey {
    pub fn public(self) -> PublicKey {
        PublicKey(self.0.map(PrivatePair::public))
    }
}
