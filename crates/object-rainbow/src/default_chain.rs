use crate::*;

#[derive(Default, Tagged, ListHashes, Topological, Clone, Copy)]
pub struct DefaultChain<A, B>(A, B);

impl<A, B: Default> DefaultChain<A, B> {
    pub fn from_first(first: A) -> Self {
        Self(first, Default::default())
    }
}

impl<A: Default, B> DefaultChain<A, B> {
    pub fn from_second(second: B) -> Self {
        Self(Default::default(), second)
    }
}

impl<A: InlineOutput + Default + PartialEq, B: ToOutput + Default> ToOutput for DefaultChain<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        if self.0 == A::default() {
            self.1.to_output(output);
        }
    }
}

impl<A: InlineOutput + Default + PartialEq, B: InlineOutput + Default> InlineOutput
    for DefaultChain<A, B>
{
}

impl<A: ParseInline<I> + PartialEq + Default, B: Parse<I> + Default, I: ParseInput> Parse<I>
    for DefaultChain<A, B>
{
    fn parse(mut input: I) -> crate::Result<Self> {
        let a = input.parse_inline()?;
        let b = if a == A::default() {
            input.parse()?
        } else {
            input.empty()?;
            Default::default()
        };
        Ok(Self(a, b))
    }
}

impl<A: ParseInline<I> + PartialEq + Default, B: ParseInline<I> + Default, I: ParseInput>
    ParseInline<I> for DefaultChain<A, B>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let a = input.parse_inline()?;
        let b = if a == A::default() {
            input.parse_inline()?
        } else {
            Default::default()
        };
        Ok(Self(a, b))
    }
}
