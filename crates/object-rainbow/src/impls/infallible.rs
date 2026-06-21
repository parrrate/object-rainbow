use typenum::U0;

use crate::*;

impl ToOutput for Infallible {
    fn to_output(&self, _: &mut impl Output) {
        match *self {}
    }
}

impl InlineOutput for Infallible {}
impl Tagged for Infallible {}

impl Size for Infallible {
    type Size = U0;
    const SIZE: usize = 0;
}

impl MaybeHasNiche for Infallible {
    type MnArray = SomeNiche<ZeroNiche<U0>>;
}

impl ByteOrd for Infallible {
    fn bytes_cmp(&self, _: &Self) -> Ordering {
        match *self {}
    }
}

impl<I: ParseInput> Parse<I> for Infallible {
    fn parse(_: I) -> crate::Result<Self> {
        Err(Error::OutOfBounds)
    }
}

impl<I: ParseInput> ParseInline<I> for Infallible {
    fn parse_inline(_: &mut I) -> crate::Result<Self> {
        Err(Error::OutOfBounds)
    }
}
