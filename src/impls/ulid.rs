use typenum::U16;
use ulid::Ulid;

use crate::*;

impl ToOutput for Ulid {
    fn to_output(&self, output: &mut dyn crate::Output) {
        self.to_bytes().to_output(output);
    }
}

impl InlineOutput for Ulid {}

impl Tagged for Ulid {}
impl ListHashes for Ulid {}
impl Topological for Ulid {}

impl<I: ParseInput> Parse<I> for Ulid {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for Ulid {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_inline().map(Self::from_bytes)
    }
}

impl Size for Ulid {
    type Size = U16;
    const SIZE: usize = 16;
}

impl MaybeHasNiche for Ulid {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}
