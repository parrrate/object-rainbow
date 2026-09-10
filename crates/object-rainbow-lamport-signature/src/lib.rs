use object_rainbow::{HASH_SIZE, InlineOutput, ToOutput};

#[derive(Debug, ToOutput, InlineOutput)]
pub struct PrivateFragment([u8; HASH_SIZE]);

#[derive(Debug, ToOutput)]
pub struct PrivatePair([PrivateFragment; 2]);
