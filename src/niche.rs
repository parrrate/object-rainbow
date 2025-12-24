use std::ops::Add;

use generic_array::{ArrayLength, GenericArray, functional::FunctionalSequence, sequence::Concat};
use typenum::{ATerm, B0, B1, Bit, Sum, TArr, U0, Unsigned};

use crate::{
    Enum, Size, SizeExt, ToOutput,
    enumkind::{EnumKind, UsizeTag},
};

pub trait MaybeHasNiche {
    type MnArray;
}

pub struct NoNiche<V>(V);
pub struct NoNiche2<A, B>(A, B);
pub struct AndNiche<V, T>(V, T);
pub struct NicheAnd<T, V>(T, V);
pub struct SomeNiche<T>(T);

pub trait Niche {
    type NeedsTag: Bit;
    type N: ArrayLength;
    fn niche() -> GenericArray<u8, Self::N>;
    type Next;
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

impl<V: Niche<NeedsTag = B1>> Niche for NoNiche<V> {
    type NeedsTag = B1;
    type N = V::N;
    fn niche() -> GenericArray<u8, Self::N> {
        V::niche()
    }
    type Next = Self;
}

impl<V: Niche<NeedsTag = B1>> MaybeNiche for NoNiche<V> {
    type N = V::N;
}

impl<U: MaybeNiche<N: Add<V::N, Output: Unsigned>>, V: Niche<NeedsTag = B1>> AsTailOf<U>
    for NoNiche<V>
{
    type WithHead = NoNiche2<U, Self>;
}

impl<V: Niche<NeedsTag = B1>, U: AsTailOf<Self>> AsHeadOf<U> for NoNiche<V> {
    type WithTail = U::WithHead;
}

impl<A: Niche<N: Add<B::N, Output: ArrayLength>>, B: Niche> Niche for NoNiche2<A, B> {
    type NeedsTag = B1;
    type N = Sum<A::N, B::N>;
    fn niche() -> GenericArray<u8, Self::N> {
        Concat::concat(A::niche(), B::niche())
    }
    type Next = NoNiche<Self>;
}

impl<A: MaybeNiche<N: Add<B::N, Output: Unsigned>>, B: MaybeNiche> MaybeNiche for NoNiche2<A, B> {
    type N = Sum<A::N, B::N>;
}

impl<
    U: MaybeNiche<N: Add<Sum<A::N, B::N>, Output: Unsigned>>,
    A: MaybeNiche<N: Add<B::N, Output: Unsigned>>,
    B: MaybeNiche,
> AsTailOf<U> for NoNiche2<A, B>
{
    type WithHead = NoNiche2<U, Self>;
}

impl<A: MaybeNiche<N: Add<B::N, Output: Unsigned>>, B: MaybeNiche, U: AsTailOf<Self>> AsHeadOf<U>
    for NoNiche2<A, B>
{
    type WithTail = U::WithHead;
}

impl<
    V: Niche<N = N, NeedsTag: NicheAuto>,
    N: ArrayLength + Add<T::N, Output: ArrayLength>,
    T: Niche,
> Niche for AndNiche<V, T>
{
    type NeedsTag = T::NeedsTag;
    type N = Sum<N, T::N>;
    fn niche() -> GenericArray<u8, Self::N> {
        Concat::concat(V::niche(), T::niche())
    }
    type Next = AndNiche<AutoNiche<V>, T::Next>;
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

impl<T: Niche<N: Add<N, Output: ArrayLength>>, V: Niche<N = N, NeedsTag: NicheAuto>, N: ArrayLength>
    Niche for NicheAnd<T, V>
{
    type NeedsTag = T::NeedsTag;
    type N = Sum<T::N, N>;
    fn niche() -> GenericArray<u8, Self::N> {
        Concat::concat(T::niche(), V::niche())
    }
    type Next = NicheAnd<T::Next, AutoNiche<V>>;
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
    type Next = T::Next;
}

impl<T: Niche<NeedsTag = B0>> MaybeNiche for SomeNiche<T> {
    type N = T::N;
}

impl<U: MaybeNiche<N: Add<T::N, Output: Unsigned>>, T: Niche<NeedsTag = B0>> AsTailOf<U>
    for SomeNiche<T>
{
    type WithHead = AndNiche<U, Self>;
}

impl<T: Niche<N: Add<U::N, Output: Unsigned>, NeedsTag = B0>, U: MaybeNiche> AsHeadOf<U>
    for SomeNiche<T>
{
    type WithTail = NicheAnd<Self, U>;
}

pub trait MnArray {
    type MaybeNiche: MaybeNiche;
}

impl MnArray for ATerm {
    type MaybeNiche = NoNiche<ZeroNoNiche<U0>>;
}

impl<T: MaybeNiche> MnArray for T {
    type MaybeNiche = T;
}

impl<T: AsHeadOf<R::MaybeNiche>, R: MnArray> MnArray for TArr<T, R> {
    type MaybeNiche = T::WithTail;
}

pub struct ZeroNoNiche<N>(N);

impl<N: ArrayLength> Niche for ZeroNoNiche<N> {
    type NeedsTag = B1;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default()
    }
    type Next = NoNiche<Self>;
}

pub struct ZeroNiche<N, Next = NoNiche<ZeroNoNiche<N>>>(N, Next);

impl<N: ArrayLength, Next> Niche for ZeroNiche<N, Next> {
    type NeedsTag = B0;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default()
    }
    type Next = Next;
}

pub struct OneNiche<N>(N);

impl<N: ArrayLength> Niche for OneNiche<N> {
    type NeedsTag = B0;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default().map(|()| 0xff)
    }
    type Next = NoNiche<ZeroNoNiche<N>>;
}

pub trait NicheOr: MaybeNiche {
    type NicheOr<U: NicheOr<N = Self::N>>: NicheOr<N = Self::N>;
    fn index(index: usize) -> usize;
}

impl<V: Niche<NeedsTag = B1>> NicheOr for NoNiche<V> {
    type NicheOr<U: NicheOr<N = Self::N>> = U;
    fn index(index: usize) -> usize {
        index + 1
    }
}

impl<A: MaybeNiche<N: Add<B::N, Output: Unsigned>>, B: MaybeNiche> NicheOr for NoNiche2<A, B> {
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

pub struct EnumNiche<E, const X: usize>(E);

impl<
    E: Enum<Kind = K>,
    K: EnumKind<Tag = T>,
    T: UsizeTag + ToOutput + Size<Size = N> + MaybeHasNiche<MnArray: MnArray<MaybeNiche = V>>,
    V: Niche<N = N>,
    N: ArrayLength,
    const X: usize,
> Niche for EnumNiche<E, X>
{
    type NeedsTag = V::NeedsTag;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        if V::NeedsTag::BOOL {
            T::from_usize(X).to_array()
        } else {
            V::niche()
        }
    }
    type Next = NoNiche<ZeroNoNiche<N>>;
}

pub trait NicheAuto: Bit {
    type Auto<T: Niche<NeedsTag = Self>>: MaybeNiche<N = T::N>;
}

impl NicheAuto for B0 {
    type Auto<T: Niche<NeedsTag = Self>> = SomeNiche<T>;
}

impl NicheAuto for B1 {
    type Auto<T: Niche<NeedsTag = Self>> = NoNiche<T>;
}

pub type AutoNiche<T> = <<T as Niche>::NeedsTag as NicheAuto>::Auto<T>;

pub type AutoEnumNiche<E, const X: usize> = AutoNiche<EnumNiche<E, X>>;

pub struct HackNiche<const X: usize>;

impl<const X: usize> Niche for HackNiche<X> {
    type NeedsTag = B1;
    type N = U0;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::default()
    }
    type Next = NoNiche<Self>;
}
