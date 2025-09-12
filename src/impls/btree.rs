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

impl<T: Inline + Ord> Object for BTreeSet<T> {
    fn parse(input: Input) -> crate::Result<Self> {
        input.collect_parse()
    }

    const TAGS: Tags = T::TAGS;
}
impl<T: ReflessInline + Ord> ReflessObject for BTreeSet<T> {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        input.collect_parse()
    }
}

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

impl<K: Inline + Ord, V: Inline> Object for BTreeMap<K, V> {
    fn parse(input: Input) -> crate::Result<Self> {
        input.collect_parse()
    }

    const TAGS: Tags = Tags(&[], &[&K::TAGS, &V::TAGS]);
}

impl<K: ReflessInline + Ord, V: ReflessInline> ReflessObject for BTreeMap<K, V> {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        input.collect_parse()
    }
}
