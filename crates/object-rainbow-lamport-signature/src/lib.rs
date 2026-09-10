use object_rainbow::{HASH_SIZE, Hash, InlineOutput, ToOutput};

const LENGTH: usize = HASH_SIZE * 8;

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PrivateFragment([u8; HASH_SIZE]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PrivatePair([PrivateFragment; 2]);

#[derive(Debug, ToOutput)]
pub struct PrivateKey([PrivatePair; LENGTH]);

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PublicFragment(Hash);
