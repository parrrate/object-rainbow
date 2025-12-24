use crate::*;

impl ToOutput for Infallible {
    fn to_output(&self, _: &mut dyn Output) {
        match *self {}
    }
}
