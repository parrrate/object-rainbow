use std::ops::Add;

use generic_array::{ArrayLength, GenericArray};
use typenum::{B0, B1, Bit, IsGreater, IsLess, ToInt, U0, U1, U2, U255, U256};

use crate::*;

pub trait TaggedOption {
    type TaggedOption;
    type Niche;
    const TAGGED_OPTION: bool = true;
    fn none_data() -> impl AsRef<[u8]>;
}

impl<T: MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>, N: Niche<NeedsTag = B>, B: Bit>
    TaggedOption for T
{
    type TaggedOption = B;
    type Niche = N;
    const TAGGED_OPTION: bool = B::BOOL;
    fn none_data() -> impl AsRef<[u8]> {
        N::niche()
    }
}

impl<T: ToOutput + TaggedOption> OptionOutput for T {
    fn to_option_output(option: Option<&Self>, output: &mut impl Output) {
        match option {
            Some(value) => {
                if T::TAGGED_OPTION {
                    0u8.to_output(output);
                }
                value.to_output(output);
            }
            None => {
                if T::TAGGED_OPTION {
                    1u8.to_output(output);
                } else {
                    if output.is_real() {
                        output.write(T::none_data().as_ref());
                    }
                }
            }
        }
    }
}

impl<T> ToOutput for Option<T>
where
    T: OptionOutput,
{
    fn to_output(&self, output: &mut impl Output) {
        T::to_option_output(self.as_ref(), output);
    }
}

impl<T: InlineOutput + TaggedOption> InlineOutput for Option<T> {}

impl<T: ByteOrd + TaggedOption<TaggedOption = B0, Niche: MinNiche>> ByteOrd for Option<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::None, Self::None) => Ordering::Equal,
            (Self::None, Self::Some(_)) => Ordering::Less,
            (Self::Some(_), Self::None) => Ordering::Greater,
            (Self::Some(a), Self::Some(b)) => a.bytes_cmp(b),
        }
    }
}

impl<T: ListHashes> ListHashes for Option<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for Option<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: Tagged> Tagged for Option<T> {
    const TAGS: Tags = T::TAGS;
}

pub trait OptionSize<N: Unsigned>: Bit {
    type Size: Unsigned;
}

impl<N: Unsigned> OptionSize<N> for B0 {
    type Size = N;
}

impl OptionSize<U0> for B1 {
    type Size = U1;
}

impl<
    T: Size<Size = N> + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B, N = N>>>,
    N: Unsigned,
    B: OptionSize<N>,
> Size for Option<T>
{
    type Size = B::Size;
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
    type Cut = B1;
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

pub trait OptionParseBit<T, I>: Bit {
    fn parse_option(input: I) -> crate::Result<Option<T>>;
}

impl<T: Parse<I>, I: ParseInput> OptionParseBit<T, I> for B1 {
    fn parse_option(mut input: I) -> crate::Result<Option<T>> {
        if input.parse_inline()? {
            input.empty()?;
            Ok(None)
        } else {
            Ok(Some(input.parse()?))
        }
    }
}

impl<
    T: ParseInline<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B0>,
    I: ParseInput,
> OptionParseBit<T, I> for B0
{
    fn parse_option(input: I) -> crate::Result<Option<T>> {
        Option::<T>::parse_as_inline(input)
    }
}

pub trait OptionParseBitInline<T, I>: OptionParseBit<T, I> {
    fn parse_option_inline(input: &mut I) -> crate::Result<Option<T>>;
}

impl<T: ParseInline<I>, I: ParseInput> OptionParseBitInline<T, I> for B1 {
    fn parse_option_inline(input: &mut I) -> crate::Result<Option<T>> {
        if input.parse_inline()? {
            Ok(None)
        } else {
            Ok(Some(input.parse_inline()?))
        }
    }
}

impl<
    T: ParseInline<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
    N: Niche<NeedsTag = B0>,
    I: ParseInput,
> OptionParseBitInline<T, I> for B0
{
    fn parse_option_inline(input: &mut I) -> crate::Result<Option<T>> {
        input.parse_compare(&N::niche())
    }
}

impl<
    T: Parse<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B>>>,
    B: OptionParseBit<T, I>,
    I: ParseInput,
> Parse<I> for Option<T>
{
    fn parse(input: I) -> crate::Result<Self> {
        B::parse_option(input)
    }
}

impl<
    T: ParseInline<I> + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B>>>,
    B: OptionParseBitInline<T, I>,
    I: ParseInput,
> ParseInline<I> for Option<T>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        B::parse_option_inline(input)
    }
}

impl Equivalent<bool> for Option<()> {
    fn into_equivalent(self) -> bool {
        self.is_none()
    }

    fn from_equivalent(object: bool) -> Self {
        (!object).then_some(())
    }
}

impl<T, U: Equivalent<T>> Equivalent<Option<T>> for Option<U> {
    fn into_equivalent(self) -> Option<T> {
        self.map(U::into_equivalent)
    }

    fn from_equivalent(option: Option<T>) -> Self {
        option.map(U::from_equivalent)
    }
}

assert_impl!(
    impl<T, E> Inline<E> for Option<T>
    where
        T: Inline<E> + MaybeHasNiche<MnArray: MaybeNiche + Niche<NeedsTag = B0>>,
        E: Clone,
    {
    }
);

assert_impl!(
    impl<T, E> Inline<E> for Option<T>
    where
        T: Inline<E> + MaybeHasNiche<MnArray: MaybeNiche + Niche<NeedsTag = B1>>,
        E: Clone,
    {
    }
);

assert_impl!(
    impl<T, E> Object<E> for Option<T>
    where
        T: Object<E> + MaybeHasNiche<MnArray: MaybeNiche + Niche<NeedsTag = B1>>,
        E: Clone,
    {
    }
);

#[test]
fn equivalent_to_bool() {
    assert_eq!(false.vec(), Option::from_equivalent(false).vec());
    assert_eq!(true.vec(), Option::from_equivalent(true).vec());
}

#[test]
fn unit_none_is_1() {
    assert_eq!(None::<()>.vec(), [1]);
}

#[test]
fn unit_none_none_is_2() {
    assert_eq!(None::<Option<()>>.vec(), [2]);
}

#[test]
fn unit_none_none_none_is_3() {
    assert_eq!(None::<Option<Option<()>>>.vec(), [3]);
}
