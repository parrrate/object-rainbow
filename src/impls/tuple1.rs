use crate::*;

impl<T: ToOutput> ToOutput for (T,) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
    }
}

impl<T: Topological> Topological for (T,) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
    }
}

impl<T: Parse<I>, I: ParseInput> Parse<I> for (T,) {
    fn parse(input: I) -> crate::Result<Self> {
        Ok((input.parse()?,))
    }
}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for (T,) {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok((input.parse_inline()?,))
    }
}

impl<T: Tagged> Tagged for (T,) {
    const TAGS: Tags = T::TAGS;
}

impl<T: Object> Object for (T,) {}

impl<T: Inline> Inline for (T,) {}

impl<T: ReflessObject> ReflessObject for (T,) {}

impl<T: ReflessInline> ReflessInline for (T,) {}

impl<T: Size> Size for (T,) {
    const SIZE: usize = T::SIZE;
    type Size = T::Size;
}
