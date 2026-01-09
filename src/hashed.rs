use std::ops::Deref;

use crate::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Tagged, Default)]
pub struct Hashed<T>(pub T);

impl<T> Deref for Hashed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ToOutput> ToOutput for Hashed<T> {
    fn to_output(&self, output: &mut dyn object_rainbow::Output) {
        self.0.data_hash().to_output(output);
    }
}

impl<T: ToOutput> InlineOutput for Hashed<T> {}

impl<T: ToOutput> Size for Hashed<T> {
    const SIZE: usize = Point::<T>::SIZE;
    type Size = <Point<T> as Size>::Size;
}
