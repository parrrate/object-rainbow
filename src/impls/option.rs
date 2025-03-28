use std::ops::Add;

use generic_array::{ArrayLength, GenericArray};
use typenum::{B0, B1, Bit};

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

impl<T: Topological> Topological for Option<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
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

pub struct OptionNiche<N>(N);

impl<N: ArrayLength> Niche for OptionNiche<N> {
    type NeedsTag = B0;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        let mut niche = GenericArray::default();
        niche[0] = 2;
        niche
    }
}

pub trait OptionNicheWrapper: Bit {
    type Wrap<N: ArrayLength>: MaybeNiche;
}

impl OptionNicheWrapper for B0 {
    type Wrap<N: ArrayLength> = NoNiche<ZeroNoNiche<N>>;
}

impl OptionNicheWrapper for B1 {
    type Wrap<N: ArrayLength> = SomeNiche<OptionNiche<N>>;
}

impl<
    T: MaybeHasNiche<
        MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B, N: Add<B, Output: ArrayLength>>>,
    >,
    B: OptionNicheWrapper,
> MaybeHasNiche for Option<T>
{
    type MnArray = B::Wrap<<Self as Size>::Size>;
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

impl<T: Inline + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>, N: Niche<NeedsTag = B>, B: Bit>
    Object for Option<T>
{
}

impl<T: Inline + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>, N: Niche<NeedsTag = B>, B: Bit>
    Inline for Option<T>
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
