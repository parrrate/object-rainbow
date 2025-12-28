use std::collections::VecDeque;

use crate::*;

impl<T: ToOutput> ToOutput for Vec<T> {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: Topological> Topological for Vec<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: ParseInline<I>, I: ParseInput> Parse<I> for Vec<T> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_vec()
    }
}

impl<T: Tagged> Tagged for Vec<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline<E>, E: 'static> Object<E> for Vec<T> {}
impl<T: ReflessInline> ReflessObject for Vec<T> {}

impl<T: ToOutput> ToOutput for VecDeque<T> {
    fn to_output(&self, output: &mut dyn Output) {
        let (l, r) = self.as_slices();
        T::slice_to_output(l, output);
        T::slice_to_output(r, output);
    }
}

impl<T: Topological> Topological for VecDeque<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: ParseInline<I>, I: ParseInput> Parse<I> for VecDeque<T> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_vec().map(From::from)
    }
}

impl<T: Tagged> Tagged for VecDeque<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline<E>, E: 'static> Object<E> for VecDeque<T> {}
impl<T: ReflessInline> ReflessObject for VecDeque<T> {}
