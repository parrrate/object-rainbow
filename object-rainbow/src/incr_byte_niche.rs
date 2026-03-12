use std::ops::Add;

use typenum::{B0, B1, IsLess, ToInt, U1, U255, U256};

use crate::*;

pub struct IncrByteNiche<K>(K);

pub trait NextNiche {
    type NextNiche;
}

pub trait WrapNext {
    type Wrap<J>;
}

impl WrapNext for B1 {
    type Wrap<J> = SomeNiche<IncrByteNiche<J>>;
}

impl WrapNext for B0 {
    type Wrap<J> = NoNiche<ZeroNoNiche<U1>>;
}

impl<K: IsLess<U256, Output = B1> + Add<B1, Output = J> + IsLess<U255, Output = B>, J, B: WrapNext>
    NextNiche for K
{
    type NextNiche = B::Wrap<J>;
}

impl<K: ToInt<u8> + NextNiche> Niche for IncrByteNiche<K> {
    type NeedsTag = typenum::B0;
    type N = U1;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::from_array([K::INT])
    }
    type Next = K::NextNiche;
}
