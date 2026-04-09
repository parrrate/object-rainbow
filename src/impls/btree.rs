use std::collections::{BTreeMap, BTreeSet};

use crate::*;

impl<T: ToOutput> ToOutput for BTreeSet<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: Topological<E>, E: 'static> Topological<E> for BTreeSet<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<E>) {
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

impl<T: Inline<E> + Ord, E: 'static> Object<E> for BTreeSet<T> {}

impl<T: ReflessInline + Ord> ReflessObject for BTreeSet<T> {}

impl<K: ToOutput, V: ToOutput> ToOutput for BTreeMap<K, V> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<K: Topological<E>, V: Topological<E>, E: 'static> Topological<E> for BTreeMap<K, V> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<E>) {
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

impl<K: Inline<E> + Ord, V: Inline<E>, E: 'static> Object<E> for BTreeMap<K, V> {}

impl<K: ReflessInline + Ord, V: ReflessInline> ReflessObject for BTreeMap<K, V> {}

impl<K: Ord> Equivalent<BTreeMap<K, ()>> for BTreeSet<K> {
    fn into_equivalent(self) -> BTreeMap<K, ()> {
        self.into_iter().map(|k| (k, ())).collect()
    }

    fn from_equivalent(object: BTreeMap<K, ()>) -> Self {
        object.into_iter().map(|(k, ())| k).collect()
    }
}
