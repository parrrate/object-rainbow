use crate::*;

impl<T: ToOutput> ToOutput for Box<T> {
    fn to_output(&self, output: &mut dyn Output) {
        (**self).to_output(output);
    }
}

impl<T: Parse<I>, I: ParseInput> Parse<I> for Box<T> {
    fn parse(input: I) -> crate::Result<Self> {
        T::parse(input).map(Self::new)
    }
}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for Box<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        T::parse_inline(input).map(Self::new)
    }
}

impl<T: Topological> Topological for Box<T> {
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

impl<T: Tagged> Tagged for Box<T> {
    const TAGS: Tags = T::TAGS;

    const HASH: Hash = T::HASH;
}

impl<T: Object> Object for Box<T> {
    fn full_hash(&self) -> Hash {
        (**self).full_hash()
    }
}

impl<T: Inline> Inline for Box<T> {}

impl<T: ReflessObject> ReflessObject for Box<T> {}

impl<T: ReflessInline> ReflessInline for Box<T> {}

impl<T: Size> Size for Box<T> {
    const SIZE: usize = T::SIZE;
    type Size = T::Size;
}

impl<T: MaybeHasNiche> MaybeHasNiche for Box<T> {
    type MnArray = T::MnArray;
}
