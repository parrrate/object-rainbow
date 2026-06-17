use std::collections::BTreeSet;

use crate::*;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Clone,
    Copy,
    Size,
    MaybeHasNiche,
    PartialEq,
    Eq,
    Default,
)]
pub struct MappedExtra<T, M = ()>(pub M, pub T);

impl<T: IntoIterator, M> IntoIterator for MappedExtra<T, M> {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.1.into_iter()
    }
}

impl<T, U: Equivalent<T>, M> Equivalent<MappedExtra<T, M>> for MappedExtra<U, M> {
    fn into_equivalent(self) -> MappedExtra<T, M> {
        MappedExtra(self.0, self.1.into_equivalent())
    }

    fn from_equivalent(mapped: MappedExtra<T, M>) -> Self {
        Self(mapped.0, mapped.1.equivalent_for())
    }
}

impl<T, M> Deref for MappedExtra<T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T, M> DerefMut for MappedExtra<T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

#[derive_for_wrapped]
pub trait MapExtra<Extra: 'static + Clone = ()> {
    type Mapped: 'static + Clone;
    fn map_extra(&self, extra: Extra) -> Self::Mapped;
}

impl<
    M: 'static + Send + Sync + Clone + ParseInline<I> + MapExtra<X, Mapped = E>,
    E: 'static + Send + Sync + Clone,
    X: 'static + Send + Sync + Clone,
    T: Parse<J>,
    I: PointInput<Extra = X, WithExtra<E> = J>,
    J: ParseInput,
> Parse<I> for MappedExtra<T, M>
{
    fn parse(mut input: I) -> crate::Result<Self> {
        let m = input.parse_inline::<M>()?;
        let x = input.extra().clone();
        let t = input.parse_extra(m.map_extra(x))?;
        Ok(Self(m, t))
    }
}

impl<
    M: 'static + Send + Sync + Clone + ParseInline<I> + MapExtra<X, Mapped = E>,
    E: 'static + Send + Sync + Clone,
    X: 'static + Send + Sync + Clone,
    T: ParseInline<J>,
    I: PointInput<Extra = X, WithExtra<E> = J>,
    J: ParseInput,
> ParseInline<I> for MappedExtra<T, M>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let m = input.parse_inline::<M>()?;
        let x = input.extra().clone();
        let t = input.parse_inline_extra(m.map_extra(x))?;
        Ok(Self(m, t))
    }
}

pub trait StaticMap<T> {
    type Mapped;
    fn static_map(x: T) -> Self::Mapped;
}

#[allow(clippy::repr_packed_without_abi)]
mod private {
    use ghost::phantom;

    #[phantom]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct SmExtra<M>;

    #[phantom]
    pub struct StaticFMap<M>;

    #[phantom]
    pub struct StaticCompose<A, B>;

    #[phantom]
    pub struct StaticCollect<C>;
}
#[doc(hidden)]
pub use self::private::*;

pub type SmExtra<M> = private::SmExtra<M>;

impl<M: StaticMap<T>, T> StaticMap<T> for SmExtra<M> {
    type Mapped = M::Mapped;

    fn static_map(x: T) -> Self::Mapped {
        M::static_map(x)
    }
}

impl<M> ToOutput for SmExtra<M> {
    fn to_output(&self, _: &mut impl Output) {}
}

impl<M> InlineOutput for SmExtra<M> {}
impl<M> Tagged for SmExtra<M> {}
impl<M> ListHashes for SmExtra<M> {}
impl<M> Topological for SmExtra<M> {}

impl<M> Size for SmExtra<M> {
    const SIZE: usize = 0;
    type Size = typenum::U0;
}

impl<M> MaybeHasNiche for SmExtra<M> {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}

impl<M> ByteOrd for SmExtra<M> {
    fn bytes_cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<M, I: ParseInput> Parse<I> for SmExtra<M> {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<M, I: ParseInput> ParseInline<I> for SmExtra<M> {
    fn parse_inline(_: &mut I) -> crate::Result<Self> {
        Ok(SmExtra)
    }
}

impl<M: StaticMap<E, Mapped: 'static + Clone>, E: 'static + Clone> MapExtra<E> for SmExtra<M> {
    type Mapped = M::Mapped;

    fn map_extra(&self, e: E) -> Self::Mapped {
        M::static_map(e)
    }
}

pub struct StaticReturn;

impl<T> StaticMap<T> for StaticReturn {
    type Mapped = T;

    fn static_map(x: T) -> Self::Mapped {
        x
    }
}

pub type Return = SmExtra<StaticReturn>;

pub struct StaticToHash;

impl<T: FullHash> StaticMap<T> for StaticToHash {
    type Mapped = Hash;

    fn static_map(x: T) -> Self::Mapped {
        x.full_hash()
    }
}

pub type ToHash = SmExtra<StaticToHash>;

pub type StaticFMap<M> = private::StaticFMap<M>;

impl<T, I: IntoIterator<Item = T>, M: StaticMap<T>> StaticMap<I> for StaticFMap<M> {
    type Mapped = Vec<M::Mapped>;

    fn static_map(it: I) -> Self::Mapped {
        it.into_iter().map(M::static_map).collect()
    }
}

pub type FMap<M> = SmExtra<StaticFMap<M>>;

pub type StaticCompose<A, B> = private::StaticCompose<A, B>;

impl<T, A: StaticMap<T>, B: StaticMap<A::Mapped>> StaticMap<T> for StaticCompose<A, B> {
    type Mapped = B::Mapped;

    fn static_map(x: T) -> Self::Mapped {
        B::static_map(A::static_map(x))
    }
}

pub type Compose<A, B> = SmExtra<StaticCompose<A, B>>;

pub struct StaticUniqueSorted;

impl<T: Ord, I: IntoIterator<Item = T>> StaticMap<I> for StaticUniqueSorted {
    type Mapped = BTreeSet<T>;

    fn static_map(it: I) -> Self::Mapped {
        it.into_iter().collect()
    }
}

pub type UniqueSorted = SmExtra<StaticUniqueSorted>;

pub struct StaticFlatten;

impl<A: IntoIterator<Item = B>, B: IntoIterator<Item = C>, C> StaticMap<A> for StaticFlatten {
    type Mapped = Vec<C>;

    fn static_map(a: A) -> Self::Mapped {
        a.into_iter().flatten().collect()
    }
}

pub type Flatten = SmExtra<StaticFlatten>;

pub type StaticCollect<C> = private::StaticCollect<C>;

impl<T, I: IntoIterator<Item = T>, C: FromIterator<T>> StaticMap<I> for StaticCollect<C> {
    type Mapped = C;

    fn static_map(it: I) -> Self::Mapped {
        C::from_iter(it)
    }
}

pub type Collect<C> = SmExtra<StaticCollect<C>>;
