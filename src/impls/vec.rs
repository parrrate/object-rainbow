use std::collections::VecDeque;

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
        T::parse_vec(input)
    }
}

impl<T: Tagged> Tagged for Vec<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline> Object for Vec<T> {}

impl<T: ReflessInline> ReflessObject for Vec<T> {}

impl<T: ToOutput> ToOutput for VecDeque<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: Topological> Topological for VecDeque<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: ParseInline<I>, I: ParseInput> Parse<I> for VecDeque<T> {
    fn parse(input: I) -> crate::Result<Self> {
        T::parse_vec(input).map(From::from)
    }
}

impl<T: Tagged> Tagged for VecDeque<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline> Object for VecDeque<T> {}

impl<T: ReflessInline> ReflessObject for VecDeque<T> {}
