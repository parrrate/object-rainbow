use signature::{Signer, Verifier};

use crate::{SigningKey, VerifyKey};

impl<V: Send + Sync + Eq + Verifier<S>, S, M: AsRef<[u8]>> VerifyKey<S, M> for V {
    type Error = signature::Error;

    fn verify(&self, signature: &S, message: &M) -> Result<(), Self::Error> {
        self.verify(message.as_ref(), signature)
    }
}

impl<
    K: AsRef<V> + Signer<S>,
    V: Send + Sync + Clone + Eq + Verifier<S>,
    S: Send + Sync,
    M: AsRef<[u8]>,
> SigningKey<V, S, M> for K
{
    fn sign(&self, message: &M) -> S {
        self.sign(message.as_ref())
    }

    fn to_verify_key(&self) -> V {
        self.as_ref().clone()
    }
}
