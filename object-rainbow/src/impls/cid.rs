use cid::CidGeneric;

use crate::*;

impl From<cid::Error> for crate::Error {
    fn from(error: cid::Error) -> Self {
        match error {
            cid::Error::Io(e) => e.into(),
            e => crate::Error::parse(e),
        }
    }
}

impl<const S: usize> ToOutput for CidGeneric<S> {
    fn to_output(&self, output: &mut impl Output) {
        self.write_bytes(output.as_write())
            .expect("unserialisable Cid is considered a bug");
    }
}

impl<const S: usize, I: ParseInput> Parse<I> for CidGeneric<S> {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<const S: usize, I: ParseInput> ParseInline<I> for CidGeneric<S> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Self::read_bytes(input.as_read()).map_err(From::from)
    }
}

/// We don't run into prefix issues because `S` is `const`
impl<const S: usize> InlineOutput for CidGeneric<S> {}
