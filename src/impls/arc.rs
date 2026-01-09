use crate::*;

impl<T: ?Sized + ToOutput> ToOutput for Arc<T> {
    fn to_output(&self, output: &mut dyn Output) {
        (**self).to_output(output);
    }
}

impl<T: ?Sized + InlineOutput> InlineOutput for Arc<T> {}

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

impl<T: ?Sized + ListPoints> ListPoints for Arc<T> {
    fn list_points(&self, f: &mut impl FnMut(Hash)) {
        (**self).list_points(f);
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn point_count(&self) -> usize {
        (**self).point_count()
    }
}

impl<T: ?Sized + Topological> Topological for Arc<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        (**self).accept_points(visitor);
    }

    fn topology(&self) -> TopoVec {
        (**self).topology()
    }
}

impl<T: ?Sized + Tagged> Tagged for Arc<T> {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}

impl<T: ?Sized + Size> Size for Arc<T> {
    const SIZE: usize = T::SIZE;
    type Size = T::Size;
}

impl<T: ?Sized + MaybeHasNiche> MaybeHasNiche for Arc<T> {
    type MnArray = T::MnArray;
}

impl<T: Clone> Equivalent<T> for Arc<T> {
    fn into_equivalent(self) -> T {
        Arc::unwrap_or_clone(self)
    }

    fn from_equivalent(object: T) -> Self {
        Arc::new(object)
    }
}
