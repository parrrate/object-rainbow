use object_rainbow::{Hash, pod};

pub trait SigningKey<Message = Hash> {
    type Signature;
    type VerifyKey: VerifyKey<Self::Signature, Message>;
    fn sign(&self, message: &Message) -> Self::Signature;
    fn to_verify_key(&self) -> Self::VerifyKey;
}

pub trait VerifyKey<Signature, Message = Hash> {
    type Error: 'static + Send + Sync + std::error::Error;
    fn verify(&self, signature: &Signature, message: &Message) -> Result<(), Self::Error>;
}

#[pod]
pub struct Signed<P, S, M> {
    public: P,
    signature: S,
    message: M,
}
