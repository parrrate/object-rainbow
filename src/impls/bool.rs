use generic_array::GenericArray;
use typenum::U1;

use crate::*;

impl ToOutput for bool {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&[*self as _])
    }
}

impl Topological for bool {
    fn accept_points(&self, _: &mut impl PointVisitor) {}
}

impl Tagged for bool {}

impl Size for bool {
    type Size = U1;
}

pub struct BoolNiche;

impl Niche for BoolNiche {
    type NeedsTag = typenum::B0;
    type N = U1;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::from_array([2])
    }
}

impl MaybeHasNiche for bool {
    type MnArray = SomeNiche<BoolNiche>;
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

impl Object for bool {}
impl Inline for bool {}
impl ReflessObject for bool {}
impl ReflessInline for bool {}
