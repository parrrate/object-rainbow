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
