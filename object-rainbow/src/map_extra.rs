use crate::*;

#[derive(Debug, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Clone, Copy)]
pub struct MappedExtra<T, M = ()>(pub M, pub T);

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
