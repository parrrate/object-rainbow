use typenum::ToUInt;

use crate::*;

impl<const N: usize, I: ParseInput> Parse<I> for [u8; N] {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<const N: usize, I: ParseInput> ParseInline<I> for [u8; N] {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_array()
    }
}

impl<const N: usize> MaybeHasNiche for [u8; N]
where
    typenum::generic_const_mappings::Const<N>: ToUInt<Output: ArrayLength>,
{
    type MnArray = NoNiche<ZeroNoNiche<typenum::generic_const_mappings::U<N>>>;
}
