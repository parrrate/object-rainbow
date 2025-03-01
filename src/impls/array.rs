use std::ops::Mul;

use typenum::ToUInt;

use crate::*;

impl<T: ToOutput, const N: usize> ToOutput for [T; N] {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: Topological, const N: usize> Topological for [T; N] {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: Tagged, const N: usize> Tagged for [T; N] {
    const TAGS: Tags = T::TAGS;
}

impl<T: Size, const N: usize> Size for [T; N]
where
    typenum::generic_const_mappings::Const<N>:
        ToUInt<Output: Unsigned> + Mul<T::Size, Output: Unsigned>,
{
    const SIZE: usize = T::SIZE * N;
    type Size = <typenum::generic_const_mappings::Const<N> as Mul<T::Size>>::Output;
}
