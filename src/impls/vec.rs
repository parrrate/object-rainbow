use std::collections::VecDeque;

use crate::*;

impl<T: InlineOutput> ToOutput for Vec<T> {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: ListHashes> ListHashes for Vec<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for Vec<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
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

impl<T: InlineOutput> ToOutput for VecDeque<T> {
    fn to_output(&self, output: &mut dyn Output) {
        let (l, r) = self.as_slices();
        T::slice_to_output(l, output);
        T::slice_to_output(r, output);
    }
}

impl<T: ListHashes> ListHashes for VecDeque<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for VecDeque<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
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
