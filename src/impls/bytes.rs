use typenum::ToUInt;

use crate::*;

impl<const N: usize, I: ParseInput> Parse<I> for [u8; N] {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<const N: usize, I: ParseInput> ParseInline<I> for [u8; N] {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_chunk().copied()
    }
}

impl<const N: usize> Size for [u8; N]
where
    typenum::generic_const_mappings::Const<N>: ToUInt<Output: Unsigned>,
{
    const SIZE: usize = N;
    type Size = typenum::generic_const_mappings::U<N>;
}

impl<const N: usize> MaybeHasNiche for [u8; N]
where
    typenum::generic_const_mappings::Const<N>: ToUInt<Output: ArrayLength>,
{
    type MnArray = NoNiche<ZeroNoNiche<typenum::generic_const_mappings::U<N>>>;
}

impl<const N: usize> Tagged for [u8; N] {}
impl<const N: usize> Object for [u8; N] {}
impl<const N: usize> Inline for [u8; N] {}
impl<const N: usize> ReflessObject for [u8; N] {}
impl<const N: usize> ReflessInline for [u8; N] {}

impl Parse<Input<'_>> for Vec<u8> {
    fn parse(input: Input) -> crate::Result<Self> {
        Ok(input.parse_all().into())
    }
}

impl Parse<ReflessInput<'_>> for Vec<u8> {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        Ok(input.parse_all().into())
    }
}

impl Tagged for Vec<u8> {}
impl Object for Vec<u8> {}
impl ReflessObject for Vec<u8> {}
