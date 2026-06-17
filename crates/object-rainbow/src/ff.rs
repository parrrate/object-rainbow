use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ListHashes, Topological, ParseAsInline,
)]
pub struct Ff;

impl ToOutput for Ff {
    fn to_output(&self, output: &mut impl Output) {
        output.write(&[0xff]);
    }
}

impl InlineOutput for Ff {}

impl<I: ParseInput> ParseInline<I> for Ff {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        if input.parse_inline::<u8>()? == 0xff {
            Ok(Self)
        } else {
            Err(Error::OutOfBounds)
        }
    }
}

impl ByteOrd for Ff {
    fn bytes_cmp(&self, Self: &Self) -> Ordering {
        Ordering::Equal
    }
}
