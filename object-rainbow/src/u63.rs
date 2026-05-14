use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U63(u64);

impl U63 {
    pub fn from_u64(n: u64) -> crate::Result<Self> {
        if n < (1 << 63) {
            Ok(Self(n))
        } else {
            Err(Error::UnsupportedLength)
        }
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
