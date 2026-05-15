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

impl Tagged for Bytes {}
impl Tagged for BytesMut {}
impl ListHashes for Bytes {}
impl ListHashes for BytesMut {}
impl Topological for Bytes {}
