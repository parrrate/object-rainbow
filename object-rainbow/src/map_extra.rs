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
    fn map_extra(x: T) -> Self::Mapped;
}

#[allow(clippy::repr_packed_without_abi)]
mod private {
    use ghost::phantom;

    #[phantom]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct SmExtra<M>;
}

pub type SmExtra<M> = private::SmExtra<M>;
#[doc(hidden)]
pub use self::private::*;

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
        M::map_extra(e)
    }
}

pub struct StaticReturn;

impl<T> StaticMap<T> for StaticReturn {
    type Mapped = T;

    fn map_extra(x: T) -> Self::Mapped {
        x
    }
}

pub type Return = SmExtra<StaticReturn>;
