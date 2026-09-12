use object_rainbow::Hash;

pub trait VerifyKey<Signature, Message = Hash> {
    type Error: 'static + Send + Sync + std::error::Error;
    fn verify(&self, signature: &Signature, message: &Message) -> Result<(), Self::Error>;
}
