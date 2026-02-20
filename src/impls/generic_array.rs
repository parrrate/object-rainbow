use std::ops::Mul;

use crate::*;

impl<T: InlineOutput, N: ArrayLength> ToOutput for GenericArray<T, N> {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: InlineOutput, N: ArrayLength> InlineOutput for GenericArray<T, N> {}

impl<T: ListHashes, N: ArrayLength> ListHashes for GenericArray<T, N> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological, N: ArrayLength> Topological for GenericArray<T, N> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: Tagged, N: ArrayLength> Tagged for GenericArray<T, N> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Size, N: ArrayLength> Size for GenericArray<T, N>
where
    N: Mul<T::Size, Output: Unsigned>,
{
    type Size = <N as Mul<T::Size>>::Output;
}

impl<T: ParseInline<I>, N: ArrayLength, I: ParseInput> Parse<I> for GenericArray<T, N> {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<T: ParseInline<I>, N: ArrayLength, I: ParseInput> ParseInline<I> for GenericArray<T, N> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_generic_array()
    }
}
