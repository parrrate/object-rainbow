use std::ops::Mul;

use typenum::ToUInt;

use crate::*;

impl<T: InlineOutput, const N: usize> ToOutput for [T; N] {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: InlineOutput, const N: usize> InlineOutput for [T; N] {}

impl<T: ListPoints, const N: usize> ListPoints for [T; N] {
    fn list_points(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_points(f);
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
        ToUInt<Output: Unsigned + Mul<T::Size, Output: Unsigned>>,
{
    const SIZE: usize = T::SIZE * N;
    type Size =
        <<typenum::generic_const_mappings::Const<N> as ToUInt>::Output as Mul<T::Size>>::Output;
}
