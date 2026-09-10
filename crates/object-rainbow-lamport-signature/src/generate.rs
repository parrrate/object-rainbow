use rand::{TryRng, rngs::SysRng};

use crate::{PrivateFragment, PrivatePair, SecretFragment};

pub trait Generate: Default {
    fn regenerate(&mut self) -> object_rainbow::Result<()>;
    fn generate() -> object_rainbow::Result<Self> {
        let mut new = Self::default();
        new.regenerate()?;
        Ok(new)
    }
}

impl Generate for SecretFragment {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        SysRng
            .try_fill_bytes(&mut self.0)
            .map_err(std::io::Error::from)?;
        Ok(())
    }
}

impl Generate for PrivateFragment {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        self.0.regenerate()
    }
}

impl Generate for PrivatePair {
    fn regenerate(&mut self) -> object_rainbow::Result<()> {
        self.0
            .iter_mut()
            .try_for_each(|fragment| fragment.regenerate())
    }
}
