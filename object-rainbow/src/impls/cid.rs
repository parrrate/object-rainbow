use cid::CidGeneric;

use crate::*;

impl<const S: usize> ToOutput for CidGeneric<S> {
    fn to_output(&self, output: &mut impl Output) {
        self.write_bytes(output.as_write())
            .expect("unserialisable Cid is considered a bug");
    }
}
