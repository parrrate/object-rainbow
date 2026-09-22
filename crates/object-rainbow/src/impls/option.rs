use typenum::{B0, B1};

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

impl<T: OptionOutput + ByteOrd + OptionPrefix> ByteOrd for Option<T>
where
    (T::OptionPrefix, T): MaybeHasNiche<MnArray: MnArray<MaybeNiche: MinNiche>>,
{
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

impl<T: OptionPrefix, N: Unsigned> Size for Option<T>
where
    (T::OptionPrefix, T):
        Size<Size = N> + MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<N = N>>>,
{
    type Size = N;
}

impl<T: OptionPrefix, N: Niche<NeedsTag = B0>> MaybeHasNiche for Option<T>
where
    (T::OptionPrefix, T): MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>,
{
    type MnArray = N::Next;
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
