use rand::{TryRng, rngs::SysRng};

use crate::{PrivateFragment, SecretFragment};

pub trait Generate: Default {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        *self = Self::generate()?;
        Ok(())
    }
    fn generate() -> object_rainbow::Result<Self>;
}

impl Generate for SecretFragment {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        SysRng
            .try_fill_bytes(&mut self.0)
            .map_err(std::io::Error::from)?;
        Ok(())
    }

    fn generate() -> object_rainbow::Result<Self> {
        let mut fragment = Self::default();
        fragment.regenerate()?;
        Ok(fragment)
    }
}

impl Generate for PrivateFragment {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        self.0.regenerate()
    }

    fn generate() -> object_rainbow::Result<Self> {
        Ok(Self(SecretFragment::generate()?))
    }
}
