use typenum::{U1, U2};

use crate::{enumkind::UsizeTag, incr_byte_niche::IncrByteNiche, *};

impl ToOutput for bool {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_real() {
            output.write(&[*self as _])
        }
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

impl ByteOrd for bool {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl UsizeTag for bool {
    fn from_usize(n: usize) -> Self {
        match n {
            0 => false,
            1 => true,
            _ => panic!("out of bounds"),
        }
    }

    fn to_usize(&self) -> usize {
        *self as _
    }

    fn try_to_usize(&self) -> Option<usize> {
        Some(self.to_usize())
    }
}

#[test]
fn none_is_2() {
    assert_eq!(None::<bool>.vec(), [2]);
}

#[test]
fn none_none_is_2() {
    assert_eq!(None::<Option<bool>>.vec(), [3]);
}
