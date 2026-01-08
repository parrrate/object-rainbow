use std::ops::Add;

use generic_array::GenericArray;
use typenum::{B0, B1, IsGreater, IsLess, ToInt, U1, U2, U255, U256};

use crate::*;

impl ToOutput for bool {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&[*self as _])
    }
}

impl Size for bool {
    type Size = U1;
}

pub struct BoolNiche<K>(K);

pub trait NextNiche {
    type NextNiche;
}

pub trait WrapNext {
    type Wrap<J>;
}

impl WrapNext for B1 {
    type Wrap<J> = SomeNiche<BoolNiche<J>>;
}

impl WrapNext for B0 {
    type Wrap<J> = NoNiche<ZeroNoNiche<U1>>;
}

impl<
    K: IsGreater<U1, Output = B1>
        + IsLess<U256, Output = B1>
        + Add<B1, Output = J>
        + IsLess<U255, Output = B>,
    J,
    B: WrapNext,
> NextNiche for K
{
    type NextNiche = B::Wrap<J>;
}

impl<K: ToInt<u8> + NextNiche> Niche for BoolNiche<K> {
    type NeedsTag = typenum::B0;
    type N = U1;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::from_array([K::INT])
    }
    type Next = K::NextNiche;
}

impl MaybeHasNiche for bool {
    type MnArray = SomeNiche<BoolNiche<U2>>;
}

impl<I: ParseInput> Parse<I> for bool {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for bool {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        match input.parse_chunk::<1>()? {
            [0] => Ok(false),
            [1] => Ok(true),
            [_] => Err(Error::OutOfBounds),
        }
    }
}

impl Tagged for bool {}
impl ListPoints for bool {}
impl Topological for bool {}
impl ReflessObject for bool {}
impl ReflessInline for bool {}

#[test]
fn none_is_2() {
    assert_eq!(None::<bool>.vec(), [2]);
}

#[test]
fn none_none_is_2() {
    assert_eq!(None::<Option<bool>>.vec(), [3]);
}
