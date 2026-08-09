use crate::*;

pub trait ParseFetch<E: 'static + Clone>: Sized {
    fn parse_fetch<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self>;
}

pub trait ParseFetchInline<E: 'static + Clone>: ParseFetch<E> {
    fn parse_fetch_inline<I: PointInput<Extra: Fetch<T = E>>>(input: &mut I) -> Result<Self>;

    fn parse_fetch_as_inline<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self> {
        input.parse_as_inline(Self::parse_fetch_inline)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchExtra<F>(pub F);

impl<
    E: 'static + Clone,
    F: ParseFetch<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> Parse<I> for FetchExtra<F>
{
    fn parse(input: I) -> crate::Result<Self> {
        F::parse_fetch(input).map(Self)
    }
}

impl<
    E: 'static + Clone,
    F: ParseFetchInline<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> ParseInline<I> for FetchExtra<F>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        F::parse_fetch_inline(input).map(Self)
    }
}
