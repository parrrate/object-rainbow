use crate::*;

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
