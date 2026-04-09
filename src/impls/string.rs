use crate::*;

impl ToOutput for String {
    fn to_output(&self, output: &mut dyn Output) {
        self.as_str().to_output(output);
    }
}

impl<I: ParseInput> Parse<I> for String {
    fn parse(input: I) -> crate::Result<Self> {
        Self::from_utf8(input.parse()?).map_err(crate::Error::parse)
    }
}

impl Tagged for String {}
impl Topological for String {}
impl<E: 'static> Object<E> for String {}
impl ReflessObject for String {}
