use rand::{TryCryptoRng, rngs::SysRng};

use crate::{PrivateFragment, PrivateKey, PrivatePair, SecretFragment};

pub trait Generate: Default {
    fn regenerate<R: ?Sized + TryCryptoRng>(&mut self, rng: &mut R) -> object_rainbow::Result<()>
    where
        std::io::Error: From<R::Error>;
    fn generate() -> object_rainbow::Result<Self> {
        let mut new = Self::default();
        new.regenerate(&mut SysRng)?;
        Ok(new)
    }
}

impl Generate for SecretFragment {
    fn regenerate<R: ?Sized + TryCryptoRng>(&mut self, rng: &mut R) -> object_rainbow::Result<()>
    where
        std::io::Error: From<R::Error>,
    {
        rng.try_fill_bytes(&mut self.0)
            .map_err(std::io::Error::from)?;
        Ok(())
    }
}

impl Generate for PrivateFragment {
    fn regenerate<R: ?Sized + TryCryptoRng>(&mut self, rng: &mut R) -> object_rainbow::Result<()>
    where
        std::io::Error: From<R::Error>,
    {
        self.0.regenerate(rng)
    }
}

impl Generate for PrivatePair {
    fn regenerate<R: ?Sized + TryCryptoRng>(&mut self, rng: &mut R) -> object_rainbow::Result<()>
    where
        std::io::Error: From<R::Error>,
    {
        self.0
            .iter_mut()
            .try_for_each(|fragment| fragment.regenerate(rng))
    }
}

impl Generate for PrivateKey {
    fn regenerate<R: ?Sized + TryCryptoRng>(&mut self, rng: &mut R) -> object_rainbow::Result<()>
    where
        std::io::Error: From<R::Error>,
    {
        self.0
            .0
            .iter_mut()
            .try_for_each(|pair| pair.regenerate(rng))
    }
}
