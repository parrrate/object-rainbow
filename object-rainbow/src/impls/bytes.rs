use bytes::Bytes;

use crate::*;

impl ToOutput for Bytes {
    fn to_output(&self, output: &mut impl Output) {
        (**self).to_output(output);
    }
}
