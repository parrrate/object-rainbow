use crate::*;

#[derive(ParseAsInline)]
pub struct Nt<T>(pub T);

impl<'a, T> IntoIterator for &'a Nt<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;

    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Nt<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;

    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for Nt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Nt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, A> ToOutput for Nt<T>
where
    for<'a> &'a T: IntoIterator<Item = A>,
    Option<A>: InlineOutput,
{
    fn to_output(&self, output: &mut impl Output) {
        for item in self {
            Some(item).to_output(output);
        }
        None.to_output(output);
    }
}

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
