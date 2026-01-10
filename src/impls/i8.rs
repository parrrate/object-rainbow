use typenum::U1;

use crate::*;

impl ToOutput for i8 {
    fn to_output(&self, output: &mut dyn crate::Output) {
        output.write(&[self.cast_unsigned()]);
    }
}

impl InlineOutput for i8 {}

impl<I: ParseInput> Parse<I> for i8 {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for i8 {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self::from_le_bytes(*input.parse_chunk::<1>()?))
    }
}

impl Size for i8 {
    type Size = U1;
    const SIZE: usize = 1;
}

impl MaybeHasNiche for i8 {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}

impl Tagged for i8 {}
impl ListHashes for i8 {}
impl Topological for i8 {}
