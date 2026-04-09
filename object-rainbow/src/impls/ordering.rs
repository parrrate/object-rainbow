use std::cmp::Ordering;

use typenum::U1;

use crate::*;

impl ToOutput for Ordering {
    fn to_output(&self, output: &mut dyn crate::Output) {
        (*self as i8).to_output(output);
    }
}

impl InlineOutput for Ordering {}
impl Tagged for Ordering {}
impl ListHashes for Ordering {}
impl Topological for Ordering {}

impl<I: ParseInput> Parse<I> for Ordering {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for Ordering {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        match input.parse_inline::<i8>()? {
            -1 => Ok(Self::Less),
            0 => Ok(Self::Equal),
            1 => Ok(Self::Greater),
            _ => Err(Error::OutOfBounds),
        }
    }
}

impl Size for Ordering {
    type Size = U1;
    const SIZE: usize = 1;
}
