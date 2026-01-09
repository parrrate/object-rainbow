use typenum::U1;

use crate::*;

impl ToOutput for u8 {
    fn to_output(&self, output: &mut dyn crate::Output) {
        output.write(&[*self]);
    }
}

impl InlineOutput for u8 {
    fn slice_to_output(slice: &[Self], output: &mut dyn crate::Output)
    where
        Self: Sized,
    {
        output.write(slice);
    }
}

impl<I: ParseInput> Parse<I> for u8 {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for u8 {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self::from_le_bytes(*input.parse_chunk::<1>()?))
    }

    fn parse_vec(input: I) -> crate::Result<Vec<Self>> {
        Ok(input.parse_all()?.into())
    }
}

impl Size for u8 {
    type Size = U1;
    const SIZE: usize = 1;
}

impl MaybeHasNiche for u8 {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}

impl Tagged for u8 {}
impl ListPoints for u8 {}
impl Topological for u8 {}
