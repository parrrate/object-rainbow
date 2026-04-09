use hex::FromHex;

use crate::HASH_SIZE;

use super::Hash;

impl FromHex for Hash {
    type Error = <[u8; HASH_SIZE] as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Self(<[u8; HASH_SIZE]>::from_hex(hex)?))
    }
}
