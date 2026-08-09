use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct AssertRefless<T>(pub T);

impl<T> ListHashes for AssertRefless<T> {}
impl<T> Topological for AssertRefless<T> {}

impl<T, I: ParseInput> Parse<I> for AssertRefless<T>
where
    T: for<'r> Parse<ReflessInput<'r>>,
{
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_refless()
    }
}

impl<T, I: ParseInput> ParseInline<I> for AssertRefless<T>
where
    T: for<'r> ParseInline<ReflessInput<'r>>,
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_refless_inline()
    }
}
