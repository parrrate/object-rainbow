use crate::*;

#[derive(ParseAsInline)]
pub struct Nt<T>(pub T);

impl<T: IntoIterator<Item = A> + FromIterator<A>, A, I: ParseInput> ParseInline<I> for Nt<T>
where
    Option<A>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut items = Vec::new();
        while let Some(item) = input.parse_inline()? {
            items.push(item);
        }
        Ok(Self(items.into_iter().collect()))
    }
}
