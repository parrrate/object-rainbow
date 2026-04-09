use crate::*;

impl<T: ?Sized + ToOutput> ToOutput for Box<T> {
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

impl<T: ?Sized + ListPoints> ListPoints for Box<T> {
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

impl<T: ?Sized + Topological> Topological for Box<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        (**self).accept_points(visitor);
    }

    fn topology(&self) -> TopoVec {
        (**self).topology()
    }
}

impl<T: ?Sized + Tagged> Tagged for Box<T> {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}

impl<T: Object<E>, E> Object<E> for Box<T> {}
impl<T: Inline<E>, E> Inline<E> for Box<T> {}
impl<T: ReflessObject> ReflessObject for Box<T> {}
impl<T: ReflessInline> ReflessInline for Box<T> {}

impl<T: ?Sized + Size> Size for Box<T> {
    const SIZE: usize = T::SIZE;
    type Size = T::Size;
}

impl<T: ?Sized + MaybeHasNiche> MaybeHasNiche for Box<T> {
    type MnArray = T::MnArray;
}

impl<T> Equivalent<T> for Box<T> {
    fn into_equivalent(self) -> T {
        *self
    }

    fn from_equivalent(object: T) -> Self {
        Box::new(object)
    }
}
