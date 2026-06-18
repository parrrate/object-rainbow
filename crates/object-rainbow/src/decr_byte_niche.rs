use typenum::{B0, B1, IsGreater, ToInt, U0, U1};

use crate::*;

pub struct DecrByteNiche<K>(K);

pub trait NextNiche {
    type NextNiche;
}

pub trait WrapNext {
    type Wrap<J>;
}

impl WrapNext for B1 {
    type Wrap<J> = SomeNiche<DecrByteNiche<J>>;
}

impl WrapNext for B0 {
    type Wrap<J> = NoNiche<ZeroNoNiche<U1>>;
}

impl<K: Sub<B, Output = J> + IsGreater<U0, Output = B>, J, B: WrapNext> NextNiche for K {
    type NextNiche = B::Wrap<J>;
}

impl<K: ToInt<u8> + NextNiche> Niche for DecrByteNiche<K> {
    type NeedsTag = typenum::B0;
    type Cut = B0;
    type N = U1;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::from_array([K::INT])
    }
    type Next = K::NextNiche;
}

impl<K: ToInt<u8>> MinNiche for DecrByteNiche<K> {}
