use std::collections::VecDeque;

use crate::{
    sequence::{PlainCollection, VecLike},
    *,
};

impl<T: InlineOutput> ToOutput for Vec<T> {
    fn to_output(&self, output: &mut impl Output) {
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

impl<T: ByteOrd + InlineOutput> ByteOrd for Vec<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.iter_bytes_cmp(other)
    }
}

impl<T> PlainCollection for Vec<T> {}
impl<T> VecLike for Vec<T> {}

impl<T: InlineOutput> ToOutput for VecDeque<T> {
    fn to_output(&self, output: &mut impl Output) {
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

impl<T: ByteOrd + InlineOutput> ByteOrd for VecDeque<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.iter_bytes_cmp(other)
    }
}

impl<T> PlainCollection for VecDeque<T> {}
impl<T> VecLike for VecDeque<T> {}

impl<T> Equivalent<Vec<T>> for VecDeque<T> {
    fn into_equivalent(self) -> Vec<T> {
        self.into()
    }

    fn from_equivalent(vec: Vec<T>) -> Self {
        vec.into()
    }
}
