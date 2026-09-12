use signature::Verifier;

use crate::VerifyKey;

impl<V: Send + Sync + Eq + Verifier<S>, S, M: AsRef<[u8]>> VerifyKey<S, M> for V {
    type Error = signature::Error;

    fn verify(&self, signature: &S, message: &M) -> Result<(), Self::Error> {
        self.verify(message.as_ref(), signature)
    }
}
