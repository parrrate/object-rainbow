use std::ops::Mul;

use typenum::ToUInt;

use crate::*;

impl<T: InlineOutput, const N: usize> ToOutput for [T; N] {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: InlineOutput, const N: usize> InlineOutput for [T; N] {}

impl<T: ListHashes, const N: usize> ListHashes for [T; N] {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological, const N: usize> Topological for [T; N] {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
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

impl<T: ParseInline<I>, const N: usize, I: ParseInput> Parse<I> for [T; N] {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<T: ParseInline<I>, const N: usize, I: ParseInput> ParseInline<I> for [T; N] {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_array()
    }
}
