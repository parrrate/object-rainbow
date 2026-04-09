use crate::*;

impl ToOutput for String {
    fn to_output(&self, output: &mut dyn Output) {
        self.as_str().to_output(output);
    }
}

impl<I: ParseInput> Parse<I> for String {
    fn parse(input: I) -> crate::Result<Self> {
        Ok(Self::from_utf8(input.parse()?)?)
    }
}

impl Tagged for String {}
impl ListPoints for String {}
impl Topological for String {}
impl<E> Object<E> for String {}
impl ReflessObject for String {}
