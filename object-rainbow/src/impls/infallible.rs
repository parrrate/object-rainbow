use typenum::U0;

use crate::*;

impl ToOutput for Infallible {
    fn to_output(&self, _: &mut impl Output) {
        match *self {}
    }
}

impl InlineOutput for Infallible {}

impl Size for Infallible {
    type Size = U0;
    const SIZE: usize = 0;
}

impl MaybeHasNiche for Infallible {
    type MnArray = SomeNiche<ZeroNiche<U0>>;
}
