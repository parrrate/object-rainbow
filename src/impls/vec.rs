use crate::*;

impl<T: ToOutput> ToOutput for Vec<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: Topological> Topological for Vec<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: ParseInline<I>, I: ParseInput> Parse<I> for Vec<T> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_collect()
    }
}

impl<T: Tagged> Tagged for Vec<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline> Object for Vec<T> {}

impl<T: ReflessInline> ReflessObject for Vec<T> {}
