use typenum::U0;

use crate::*;

impl ToOutput for Infallible {
    fn to_output(&self, _: &mut dyn Output) {
        match *self {}
    }
}

impl Size for Infallible {
    type Size = U0;
    const SIZE: usize = 0;
}
