use rand::{TryRng, rngs::SysRng};

use crate::{PrivateFragment, SecretFragment};

impl SecretFragment {
    pub fn regenerate(&mut self) -> object_rainbow::Result<()> {
        SysRng
            .try_fill_bytes(&mut self.0)
            .map_err(std::io::Error::from)?;
        Ok(())
    }

    pub fn generate() -> object_rainbow::Result<Self> {
        let mut fragment = Self::default();
        fragment.regenerate()?;
        Ok(fragment)
    }
}

impl PrivateFragment {
    pub fn generate() -> object_rainbow::Result<Self> {
        Ok(Self(SecretFragment::generate()?))
    }
}
