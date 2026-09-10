use object_rainbow::HASH_SIZE;
use rand::{TryRng, rngs::SysRng};

use crate::{PrivateFragment, SecretFragment};

impl SecretFragment {
    pub fn generate() -> object_rainbow::Result<Self> {
        let mut bytes = [0u8; HASH_SIZE];
        SysRng
            .try_fill_bytes(&mut bytes)
            .map_err(std::io::Error::from)?;
        Ok(Self(bytes))
    }
}

impl PrivateFragment {
    pub fn generate() -> object_rainbow::Result<Self> {
        Ok(Self(SecretFragment::generate()?))
    }
}
