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
