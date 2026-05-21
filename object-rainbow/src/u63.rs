use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ParseAsInline)]
pub struct U63(u64);

impl U63 {
    pub fn from_u64(n: u64) -> crate::Result<Self> {
        if n < ((1 << 63) + 128) {
            Ok(Self(n))
        } else {
            Err(Error::UnsupportedLength)
        }
    }

    pub fn as_usize(self) -> crate::Result<usize> {
        self.0.try_into().map_err(|_| Error::UnsupportedLength)
    }

    pub fn len_of<T>(v: &[T]) -> Self {
        Self::from_u64(v.len() as _).expect("this is unreasonable to store in memory")
    }
}

impl ToOutput for U63 {
    fn to_output(&self, output: &mut impl Output) {
        if self.0 < 128 {
            (self.0 as u8).to_output(output);
        } else {
            let mut bytes = self.0.to_be_bytes();
            bytes[0] ^= 128;
            bytes.to_output(output);
        }
    }
}

impl InlineOutput for U63 {}
impl Tagged for U63 {}
impl ListHashes for U63 {}
impl Topological for U63 {}

impl<I: ParseInput> ParseInline<I> for U63 {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut bytes = [0; 8];
        input.read(&mut bytes[..1])?;
        Ok(Self(if bytes[0] & 128 == 0 {
            bytes[0] as u64
        } else {
            bytes[0] ^= 128;
            input.read(&mut bytes[1..])?;
            let n = u64::from_be_bytes(bytes);
            if n < 128 {
                return Err(Error::OutOfBounds);
            }
            n
        }))
    }
}

impl ByteOrd for U63 {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl SignificantLength for U63 {}

#[test]
fn reparse_u63_100_000() -> crate::Result<()> {
    (0..100_000).try_for_each(|n| {
        assert_eq!(U63::from_u64(n)?.reparse()?.0, n);
        Ok(())
    })
}
