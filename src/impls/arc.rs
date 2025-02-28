use crate::*;

impl<T: ToOutput> ToOutput for Arc<T> {
    fn to_output(&self, output: &mut dyn Output) {
        (**self).to_output(output);
    }
}

impl<T: Parse<I>, I: ParseInput> Parse<I> for Arc<T> {
    fn parse(input: I) -> crate::Result<Self> {
        T::parse(input).map(Self::new)
    }
}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for Arc<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        T::parse_inline(input).map(Self::new)
    }
}

impl<T: Topological> Topological for Arc<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        (**self).accept_points(visitor);
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn topology(&self) -> TopoVec {
        (**self).topology()
    }
}

impl<T: Tagged> Tagged for Arc<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Object> Object for Arc<T> {
    fn full_hash(&self) -> Hash {
        (**self).full_hash()
    }
}

impl<T: Inline> Inline for Arc<T> {}

impl<T: ReflessObject> ReflessObject for Arc<T> {}

impl<T: ReflessInline> ReflessInline for Arc<T> {}

impl<T: Size> Size for Arc<T> {
    const SIZE: usize = T::SIZE;
}
