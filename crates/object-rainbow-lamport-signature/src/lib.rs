use object_rainbow::{HASH_SIZE, ToOutput};

#[derive(Debug, ToOutput)]
pub struct PrivateFragment([u8; HASH_SIZE]);
