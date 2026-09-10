use object_rainbow::{HASH_SIZE, Hash, InlineOutput, ToOutput};

const LENGTH: usize = HASH_SIZE * 8;

#[derive(Debug, ToOutput, InlineOutput, Clone, Copy)]
pub struct PrivateFragment([u8; HASH_SIZE]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PrivatePair([PrivateFragment; 2]);

#[derive(Debug, ToOutput)]
pub struct PrivateKey([PrivatePair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PublicFragment(Hash);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PublicPair([PublicFragment; 2]);

#[derive(Debug, ToOutput)]
pub struct PublicKey([PublicPair; LENGTH]);

impl PrivateFragment {
    pub fn public(&self) -> PublicFragment {
        PublicFragment(self.data_hash())
    }
}
