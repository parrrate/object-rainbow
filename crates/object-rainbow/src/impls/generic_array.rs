use std::ops::Mul;

use typenum::{B0, B1, IsGreater, U0, U1};

use crate::{niche_cut::NicheCut, *};

impl<T: InlineOutput, N: ArrayLength> ToOutput for GenericArray<T, N> {
    fn to_output(&self, output: &mut impl Output) {
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

impl<T: ByteOrd + InlineOutput, N: ArrayLength> ByteOrd for GenericArray<T, N> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.iter_bytes_cmp(other)
    }
}

pub trait MoreThan1<T, N> {
    type MnArray;
}

impl<T, N> MoreThan1<T, N> for B1
where
    (T, NicheCut): MaybeHasNiche,
{
    type MnArray = <(T, NicheCut) as MaybeHasNiche>::MnArray;
}

impl<T: MaybeHasNiche, N> MoreThan1<T, N> for B0 {
    type MnArray = T::MnArray;
}

pub trait MoreThan0<T, N> {
    type MnArray;
}

impl<T, N: IsGreater<U1, Output = B>, B: MoreThan1<T, N>> MoreThan0<T, N> for B1 {
    type MnArray = B::MnArray;
}

impl<T, N> MoreThan0<T, N> for B0 {
    type MnArray = NoNiche<ZeroNoNiche<U0>>;
}

impl<T, N: ArrayLength + IsGreater<U0, Output = B>, B: MoreThan0<T, N>> MaybeHasNiche
    for GenericArray<T, N>
{
    type MnArray = B::MnArray;
}
