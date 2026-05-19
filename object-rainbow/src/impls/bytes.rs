use bytes::{Bytes, BytesMut};

use crate::*;

impl ToOutput for Bytes {
    fn to_output(&self, output: &mut impl Output) {
        (**self).to_output(output);
    }
}

impl ToOutput for BytesMut {
    fn to_output(&self, output: &mut impl Output) {
        (**self).to_output(output);
    }
}

impl<I: ParseInput> Parse<I> for Bytes {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_all().map(Into::into).map(Vec::into)
    }
}

impl<I: ParseInput> Parse<I> for BytesMut {
    fn parse(input: I) -> crate::Result<Self> {
        input
            .parse_all()
            .map(Into::into)
            .map(Vec::into)
            .map(Bytes::into)
    }
}

impl Tagged for Bytes {}
impl Tagged for BytesMut {}
impl ListHashes for Bytes {}
impl ListHashes for BytesMut {}
impl Topological for Bytes {}
impl Topological for BytesMut {}
impl MaybeHasNiche for Bytes {
    type MnArray = NoNiche<NicheForUnsized>;
}
impl MaybeHasNiche for BytesMut {
    type MnArray = NoNiche<NicheForUnsized>;
}

impl ByteOrdered for Bytes {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}
