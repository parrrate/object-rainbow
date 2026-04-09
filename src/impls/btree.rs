use std::collections::{BTreeMap, BTreeSet};

use crate::*;

impl<T: ToOutput> ToOutput for BTreeSet<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: Topological> Topological for BTreeSet<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: ParseInline<I> + Ord, I: ParseInput> Parse<I> for BTreeSet<T> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_collect()
    }
}

impl<T: Tagged> Tagged for BTreeSet<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Inline + Ord> Object for BTreeSet<T> {}

impl<T: ReflessInline + Ord> ReflessObject for BTreeSet<T> {}

impl<K: ToOutput, V: ToOutput> ToOutput for BTreeMap<K, V> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<K: Topological, V: Topological> Topological for BTreeMap<K, V> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<K: ParseInline<I> + Ord, V: ParseInline<I>, I: ParseInput> Parse<I> for BTreeMap<K, V> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_collect()
    }
}

impl<K: Tagged, V: Tagged> Tagged for BTreeMap<K, V> {
    const TAGS: Tags = Tags(&[], &[&K::TAGS, &V::TAGS]);
}

impl<K: Inline + Ord, V: Inline> Object for BTreeMap<K, V> {}

impl<K: ReflessInline + Ord, V: ReflessInline> ReflessObject for BTreeMap<K, V> {}
