use std::ops::Add;

use generic_array::{ArrayLength, GenericArray, sequence::Concat};
use typenum::{ATerm, B0, B1, Bit, Sum, TArr, U0, Unsigned};

pub trait MaybeHasNiche {
    type MnArray;
}

pub struct NoNiche<N>(N);
pub struct AndNiche<V, T>(V, T);
pub struct NicheAnd<T, V>(T, V);
pub struct SomeNiche<T>(T);

pub trait Niche {
    type NeedsTag: Bit;
    type N: ArrayLength;
    fn niche() -> GenericArray<u8, Self::N>;
}

pub trait MaybeNiche {
    type N: Unsigned;
}

pub trait AsTailOf<U: MaybeNiche>: MaybeNiche {
    type WithHead: MaybeNiche;
}

pub trait AsHeadOf<U: MaybeNiche>: MaybeNiche {
    type WithTail: MaybeNiche;
}

impl<N: ArrayLength> Niche for NoNiche<N> {
    type NeedsTag = B1;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default()
    }
}

impl<N: Unsigned> MaybeNiche for NoNiche<N> {
    type N = N;
}

impl<U: MaybeNiche<N: Add<N, Output: Unsigned>>, N: Unsigned> AsTailOf<U> for NoNiche<N> {
    type WithHead = NoNiche<Sum<U::N, N>>;
}

impl<N: Unsigned, U: AsTailOf<Self>> AsHeadOf<U> for NoNiche<N> {
    type WithTail = U::WithHead;
}

impl<V: Niche<N = N>, N: ArrayLength + Add<T::N, Output: ArrayLength>, T: Niche> Niche
    for AndNiche<V, T>
{
    type NeedsTag = T::NeedsTag;
    type N = Sum<N, T::N>;

    fn niche() -> GenericArray<u8, Self::N> {
        Concat::concat(V::niche(), T::niche())
    }
}

impl<V: MaybeNiche<N = N>, N: Unsigned, T: MaybeNiche> MaybeNiche for AndNiche<V, T>
where
    N: Add<T::N, Output: Unsigned>,
{
    type N = Sum<N, T::N>;
}

impl<
    U: MaybeNiche<N: Add<Sum<N, T::N>, Output: Unsigned>>,
    V: MaybeNiche<N = N>,
    N: Unsigned,
    T: MaybeNiche,
> AsTailOf<U> for AndNiche<V, T>
where
    N: Add<T::N, Output: Unsigned>,
{
    type WithHead = AndNiche<U, Self>;
}

impl<V: MaybeNiche<N = N>, N: Unsigned, T: MaybeNiche, U: MaybeNiche> AsHeadOf<U> for AndNiche<V, T>
where
    N: Add<T::N, Output: Unsigned>,
    Sum<N, T::N>: Add<U::N, Output: Unsigned>,
{
    type WithTail = NicheAnd<Self, U>;
}

impl<T: Niche<N: Add<N, Output: ArrayLength>>, V: Niche<N = N>, N: ArrayLength> Niche
    for NicheAnd<T, V>
{
    type NeedsTag = T::NeedsTag;
    type N = Sum<T::N, N>;

    fn niche() -> GenericArray<u8, Self::N> {
        Concat::concat(T::niche(), V::niche())
    }
}

impl<T: MaybeNiche<N: Add<N, Output: Unsigned>>, V: MaybeNiche<N = N>, N: Unsigned> MaybeNiche
    for NicheAnd<T, V>
{
    type N = Sum<T::N, N>;
}

impl<
    U: MaybeNiche<N: Add<Sum<T::N, N>, Output: Unsigned>>,
    T: MaybeNiche<N: Add<N, Output: Unsigned>>,
    V: MaybeNiche<N = N>,
    N: Unsigned,
> AsTailOf<U> for NicheAnd<T, V>
{
    type WithHead = AndNiche<U, Self>;
}

impl<T: MaybeNiche<N: Add<N, Output: Unsigned>>, V: MaybeNiche<N = N>, N: Unsigned, U: MaybeNiche>
    AsHeadOf<U> for NicheAnd<T, V>
where
    Sum<T::N, N>: Add<U::N, Output: Unsigned>,
{
    type WithTail = NicheAnd<Self, U>;
}

impl<T: Niche<NeedsTag = B0>> Niche for SomeNiche<T> {
    type NeedsTag = T::NeedsTag;
    type N = T::N;
    fn niche() -> GenericArray<u8, Self::N> {
        T::niche()
    }
}

impl<T: Niche<NeedsTag = B0>> MaybeNiche for SomeNiche<T> {
    type N = T::N;
}

impl<U: MaybeNiche<N: Add<T::N, Output: Unsigned>>, T: Niche<NeedsTag = B0>> AsTailOf<U>
    for SomeNiche<T>
{
    type WithHead = AndNiche<U, SomeNiche<T>>;
}

impl<T: Niche<N: Add<U::N, Output: Unsigned>, NeedsTag = B0>, U: MaybeNiche> AsHeadOf<U>
    for SomeNiche<T>
{
    type WithTail = NicheAnd<SomeNiche<T>, U>;
}

pub trait MnArray {
    type MaybeNiche: MaybeNiche;
}

impl MnArray for ATerm {
    type MaybeNiche = NoNiche<U0>;
}

impl<T: MaybeNiche> MnArray for T {
    type MaybeNiche = T;
}

impl<T: AsHeadOf<R::MaybeNiche>, R: MnArray> MnArray for TArr<T, R> {
    type MaybeNiche = T::WithTail;
}

pub struct ZeroNiche<N>(N);

impl<N: ArrayLength> Niche for ZeroNiche<N> {
    type NeedsTag = B0;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default()
    }
}

pub trait NicheOr: MaybeNiche {
    type NicheOr<U: NicheOr<N = Self::N>>: NicheOr<N = Self::N>;
    fn index(index: usize) -> usize;
}

impl<N: Unsigned> NicheOr for NoNiche<N> {
    type NicheOr<U: NicheOr<N = Self::N>> = U;
    fn index(index: usize) -> usize {
        index + 1
    }
}

impl<V: MaybeNiche<N = N>, N: Unsigned + Add<T::N, Output: Unsigned>, T: MaybeNiche> NicheOr
    for AndNiche<V, T>
{
    type NicheOr<U: NicheOr<N = Self::N>> = Self;
    fn index(_: usize) -> usize {
        0
    }
}

impl<T: MaybeNiche<N: Add<N, Output: Unsigned>>, V: MaybeNiche<N = N>, N: Unsigned> NicheOr
    for NicheAnd<T, V>
{
    type NicheOr<U: NicheOr<N = Self::N>> = Self;
    fn index(_: usize) -> usize {
        0
    }
}

impl<T: Niche<NeedsTag = B0>> NicheOr for SomeNiche<T> {
    type NicheOr<U: NicheOr<N = Self::N>> = Self;
    fn index(_: usize) -> usize {
        0
    }
}

pub trait NicheFoldOr {
    type Or: NicheOr;
    fn index() -> usize;
}

impl<T: MnArray<MaybeNiche: NicheOr>> NicheFoldOr for TArr<T, ATerm> {
    type Or = T::MaybeNiche;
    fn index() -> usize {
        0
    }
}

impl<T: NicheOr, A: NicheFoldOr<Or: MaybeNiche<N = T::N>>> NicheFoldOr for TArr<T, A> {
    type Or = T::NicheOr<A::Or>;
    fn index() -> usize {
        T::index(A::index())
    }
}

pub struct NicheFoldOrArray<T>(T);

impl<T: NicheFoldOr> MnArray for NicheFoldOrArray<T> {
    type MaybeNiche = T::Or;
}
