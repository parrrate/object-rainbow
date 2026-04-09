use typenum::{U1, U2};

use crate::{incr_byte_niche::IncrByteNiche, *};

impl ToOutput for bool {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&[*self as _])
    }
}

impl InlineOutput for bool {}

impl Size for bool {
    type Size = U1;
}

impl MaybeHasNiche for bool {
    type MnArray = SomeNiche<IncrByteNiche<U2>>;
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
impl ListHashes for bool {}
impl Topological for bool {}

#[test]
fn none_is_2() {
    assert_eq!(None::<bool>.vec(), [2]);
}

#[test]
fn none_none_is_2() {
    assert_eq!(None::<Option<bool>>.vec(), [3]);
}
