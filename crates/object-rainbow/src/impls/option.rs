use std::ops::Add;

use generic_array::{ArrayLength, GenericArray};
use typenum::{B0, B1, Bit, IsGreater, IsLess, ToInt, U0, U1, U2, U255, U256};

use crate::{ff::Ff, niche_cut::NicheCut, *};

pub trait OptionPrefix {
    type OptionPrefix: Monostate + InlineOutput + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche>>;
}

pub trait OptionPrefixBit {
    type OptionPrefix: Monostate + InlineOutput + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche>>;
}

impl OptionPrefixBit for B0 {
    type OptionPrefix = ();
}

impl OptionPrefixBit for B1 {
    type OptionPrefix = (Ff, NicheCut);
}

impl<T: MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B>>>, B: OptionPrefixBit>
    OptionPrefix for T
{
    type OptionPrefix = B::OptionPrefix;
}

pub trait TaggedOption {
    type TaggedOption;
    type Niche;
    const TAGGED_OPTION: bool = true;
    fn none_data() -> impl AsRef<[u8]>;
    fn none_output(output: &mut (impl ?Sized + Output)) {
        if output.is_real() {
            output.write(Self::none_data().as_ref());
        }
    }
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

impl<T: ToOutput + OptionPrefix, N: Niche<NeedsTag = B0>> OptionOutput for T
where
    (T::OptionPrefix, T): MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
{
    fn to_option_output(option: Option<&Self>, output: &mut (impl ?Sized + Output)) {
        match option {
            Some(value) => {
                (T::OptionPrefix::default(), value).to_output(output);
            }
            None => {
                N::niche().to_output(output);
            }
        }
    }
}

impl<T: OptionOutput> ToOutput for Option<T> {
    fn to_output(&self, output: &mut (impl ?Sized + Output)) {
        T::to_option_output(self.as_ref(), output);
    }
}

impl<T: OptionOutput + InlineOutput> InlineOutput for Option<T> {}

impl<T: OptionOutput + ByteOrd + TaggedOption<Niche: MinNiche>> ByteOrd for Option<T> {
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
    fn list_hashes(&self, f: &mut (impl ?Sized + FnMut(Hash))) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for Option<T> {
    fn traverse(&self, visitor: &mut (impl ?Sized + PointVisitor)) {
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
        niche[0] = u8::MAX - K::INT;
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
    T: Parse<I> + OptionPrefix<OptionPrefix: ParseInline<I>>,
    I: ParseInput,
    N: Niche<NeedsTag = B0>,
> OptionParse<I> for T
where
    (T::OptionPrefix, T): MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
{
    fn parse_option(input: I) -> crate::Result<Option<Self>> {
        Ok(input
            .parse_compare::<(T::OptionPrefix, T)>(&N::niche())?
            .map(|(_, object)| object))
    }
}

impl<T: OptionParse<I>, I: ParseInput> Parse<I> for Option<T> {
    fn parse(input: I) -> crate::Result<Self> {
        T::parse_option(input)
    }
}

impl<
    T: ParseInline<I> + OptionPrefix<OptionPrefix: ParseInline<I>>,
    I: ParseInput,
    N: Niche<NeedsTag = B0>,
> OptionParseInline<I> for T
where
    (T::OptionPrefix, T): MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
{
    fn parse_option_inline(input: &mut I) -> crate::Result<Option<Self>> {
        Ok(input
            .parse_compare_inline::<(T::OptionPrefix, T)>(&N::niche())?
            .map(|(_, object)| object))
    }
}

impl<T: OptionParseInline<I>, I: ParseInput> ParseInline<I> for Option<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        T::parse_option_inline(input)
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
        ((), T): MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B0>>>,
    {
    }
);

assert_impl!(
    impl<T, E> Inline<E> for Option<T>
    where
        T: Inline<E> + MaybeHasNiche<MnArray: MaybeNiche + Niche<NeedsTag = B1>>,
        E: Clone,
        ((Ff, NicheCut), T): MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B0>>>,
    {
    }
);

#[test]
fn unit_none_is_254() {
    assert_eq!(None::<()>.vec(), [254]);
}

#[test]
fn unit_none_none_is_253() {
    assert_eq!(None::<Option<()>>.vec(), [253]);
}

#[test]
fn unit_none_none_none_is_252() {
    assert_eq!(None::<Option<Option<()>>>.vec(), [252]);
}
