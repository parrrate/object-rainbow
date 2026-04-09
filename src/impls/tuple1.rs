use crate::*;

impl<T: ToOutput> ToOutput for (T,) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
    }
}

impl<T: ListPoints> ListPoints for (T,) {
    fn list_points(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_points(f);
    }

    fn topology_hash(&self) -> Hash {
        self.0.topology_hash()
    }

    fn point_count(&self) -> usize {
        self.0.point_count()
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

impl<T: Object<E>, E> Object<E> for (T,) {}
impl<T: Inline<E>, E> Inline<E> for (T,) {}
impl<T: ReflessObject> ReflessObject for (T,) {}
impl<T: ReflessInline> ReflessInline for (T,) {}

impl<T: Size> Size for (T,) {
    const SIZE: usize = T::SIZE;
    type Size = T::Size;
}

impl<T: MaybeHasNiche> MaybeHasNiche for (T,) {
    type MnArray = T::MnArray;
}

impl<T> Equivalent<T> for (T,) {
    fn into_equivalent(self) -> T {
        self.0
    }

    fn from_equivalent(object: T) -> Self {
        (object,)
    }
}
