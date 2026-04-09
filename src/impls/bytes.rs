use crate::*;

impl<const N: usize> ToOutput for [u8; N] {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self);
    }
}

impl<const N: usize> Parse<ReflessInput<'_>> for [u8; N] {
    fn parse(input: ReflessInput<'_>) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<const N: usize> ParseInline<ReflessInput<'_>> for [u8; N] {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        input.parse_chunk().copied()
    }
}

impl<const N: usize> Tagged for [u8; N] {}

impl<const N: usize> ReflessObject for [u8; N] {}

impl<const N: usize> ReflessInline for [u8; N] {}

impl<T: Size, const N: usize> Size for [T; N] {
    const SIZE: usize = T::SIZE * N;
}

impl<const N: usize> Size for [u8; N] {
    const SIZE: usize = N;
}

impl ToOutput for Vec<u8> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self);
    }
}

impl Parse<ReflessInput<'_>> for Vec<u8> {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        Ok(input.parse_all()?.into())
    }
}

impl Tagged for Vec<u8> {}

impl ReflessObject for Vec<u8> {}
