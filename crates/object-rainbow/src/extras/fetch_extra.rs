use crate::*;

pub trait ParseDelayedOpaque<E: 'static + Clone>: Sized {
    fn parse_delayed_opaque<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self>;
}

pub trait ParseDelayedOpaqueInline<E: 'static + Clone>: ParseDelayedOpaque<E> {
    fn parse_delayed_opaque_inline<I: PointInput<Extra: Fetch<T = E>>>(
        input: &mut I,
    ) -> Result<Self>;

    fn parse_delayed_opaque_as_inline<I: PointInput<Extra: Fetch<T = E>>>(
        input: I,
    ) -> Result<Self> {
        input.parse_as_inline(Self::parse_delayed_opaque_inline)
    }
}

pub struct DelayedOpaque<F>(pub F);

impl<
    E: 'static + Clone,
    F: ParseDelayedOpaque<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> Parse<I> for DelayedOpaque<F>
{
    fn parse(input: I) -> crate::Result<Self> {
        F::parse_delayed_opaque(input).map(Self)
    }
}

impl<
    E: 'static + Clone,
    F: ParseDelayedOpaqueInline<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> ParseInline<I> for DelayedOpaque<F>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        F::parse_delayed_opaque_inline(input).map(Self)
    }
}
