use crate::*;

/// Parses `Extra`, then provides it to `T`'s parser.
#[derive(Debug, ToOutput, InlineOutput, Tagged, ListHashes, Topological, Clone, Copy)]
pub struct InlineExtra<T, Extra = ()>(pub Extra, pub T);

impl<
    E: 'static + Send + Sync + Clone + ParseInline<I>,
    X: 'static + Send + Sync + Clone,
    T: Parse<J>,
    I: PointInput<Extra = X, WithExtra<(E, X)> = J>,
    J: ParseInput,
> Parse<I> for InlineExtra<T, E>
{
    fn parse(mut input: I) -> crate::Result<Self> {
        let e = input.parse_inline::<E>()?;
        let x = input.extra().clone();
        let t = input.parse_extra((e.clone(), x))?;
        Ok(Self(e, t))
    }
}

impl<
    E: 'static + Send + Sync + Clone + ParseInline<I>,
    X: 'static + Send + Sync + Clone,
    T: ParseInline<J>,
    I: PointInput<Extra = X, WithExtra<(E, X)> = J>,
    J: ParseInput,
> ParseInline<I> for InlineExtra<T, E>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let e = input.parse_inline::<E>()?;
        let x = input.extra().clone();
        let t = input.parse_inline_extra((e.clone(), x))?;
        Ok(Self(e, t))
    }
}
