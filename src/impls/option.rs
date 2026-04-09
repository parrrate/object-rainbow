use std::ops::Add;

use generic_array::{ArrayLength, GenericArray};
use typenum::{B0, B1, Bit, IsGreater, IsLess, ToInt, U1, U2, U255, U256};

use crate::*;

impl<T: ToOutput + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>, N: Niche<NeedsTag = B>, B: Bit>
    ToOutput for Option<T>
{
    fn to_output(&self, output: &mut dyn Output) {
        match self {
            Some(value) => {
                if B::BOOL {
                    output.write(&[0]);
                }
                value.to_output(output);
            }
            None => {
                if B::BOOL {
                    output.write(&[1]);
                }
                output.write(N::niche().as_slice());
            }
        }
    }
}

impl<T: Topological<E>, E: 'static> Topological<E> for Option<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<E>) {
        self.iter_accept_points(visitor);
    }
}

impl<T: Tagged> Tagged for Option<T> {
    const TAGS: Tags = T::TAGS;
}

impl<
    T: MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B, N: Add<B, Output = N>>>>,
    B: Bit,
    N: Unsigned,
> Size for Option<T>
{
    type Size = N;
}

pub struct UnspecifiedOptionNiche;

pub struct OptionNiche<N, K>(N, K);

pub trait NextNiche {
    type NextNiche<N: ArrayLength>;
}

pub trait WrapNext {
    type Wrap<N: ArrayLength, J>;
}

impl WrapNext for B1 {
    type Wrap<N: ArrayLength, J> = SomeNiche<OptionNiche<N, J>>;
}

impl WrapNext for B0 {
    type Wrap<N: ArrayLength, J> = UnspecifiedOptionNiche;
}

impl<
    K: IsGreater<U1, Output = B1>
        + IsLess<U256, Output = B1>
        + Add<B1, Output = J>
        + IsLess<U255, Output = B>,
    J,
    B: WrapNext,
> NextNiche for K
{
    type NextNiche<N: ArrayLength> = B::Wrap<N, J>;
}

impl<N: ArrayLength, K: ToInt<u8> + NextNiche> Niche for OptionNiche<N, K> {
    type NeedsTag = B0;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        let mut niche = GenericArray::default();
        niche[0] = K::INT;
        niche
    }
    type Next = K::NextNiche<N>;
}

pub trait OptionNicheWrapper: Bit {
    type Wrap<Mn: Niche<NeedsTag = Self, N: Add<Self, Output: ArrayLength>>>;
}

impl OptionNicheWrapper for B0 {
    type Wrap<Mn: Niche<NeedsTag = Self, N: Add<Self, Output: ArrayLength>>> = Mn::Next;
}

impl OptionNicheWrapper for B1 {
    type Wrap<Mn: Niche<NeedsTag = Self, N: Add<Self, Output: ArrayLength>>> =
        SomeNiche<OptionNiche<<<Mn as Niche>::N as Add<Self>>::Output, U2>>;
}

impl<
    T: MaybeHasNiche<MnArray: MnArray<MaybeNiche = Mn>>,
    Mn: Niche<NeedsTag = B, N: Add<B, Output: ArrayLength>>,
    B: OptionNicheWrapper,
> MaybeHasNiche for Option<T>
{
    type MnArray = B::Wrap<Mn>;
}

impl<
    T: ParseInline<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
    I: ParseInput,
> Parse<I> for Option<T>
{
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<
    T: ParseInline<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
    I: ParseInput,
> ParseInline<I> for Option<T>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        if B::BOOL {
            if input.parse_inline()? {
                Ok(None)
            } else {
                Ok(Some(input.parse_inline()?))
            }
        } else {
            input.parse_compare(N::N::USIZE, &N::niche())
        }
    }
}

impl<
    T: Inline<E> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
    E: 'static,
> Object<E> for Option<T>
{
}

impl<
    T: Inline<E> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
    E: 'static,
> Inline<E> for Option<T>
{
}

impl<
    T: ReflessInline + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
> ReflessObject for Option<T>
{
}

impl<
    T: ReflessInline + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B>,
    B: Bit,
> ReflessInline for Option<T>
{
}

impl Equivalent<bool> for Option<()> {
    fn into_equivalent(self) -> bool {
        self.is_none()
    }

    fn from_equivalent(object: bool) -> Self {
        (!object).then_some(())
    }
}

#[test]
fn equivalent() {
    assert_eq!(
        false.output::<Vec<u8>>(),
        Option::from_equivalent(false).output::<Vec<u8>>(),
    );
    assert_eq!(
        true.output::<Vec<u8>>(),
        Option::from_equivalent(true).output::<Vec<u8>>(),
    );
}
