use std::collections::{BTreeMap, BTreeSet};

use crate::*;

impl<T: InlineOutput> ToOutput for BTreeSet<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<T: ListHashes> ListHashes for BTreeSet<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for BTreeSet<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
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

impl<K: InlineOutput, V: InlineOutput> ToOutput for BTreeMap<K, V> {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
    }
}

impl<K: ListHashes, V: ListHashes> ListHashes for BTreeMap<K, V> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<K: Topological, V: Topological> Topological for BTreeMap<K, V> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
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

impl<K: Ord> Equivalent<BTreeMap<K, ()>> for BTreeSet<K> {
    fn into_equivalent(self) -> BTreeMap<K, ()> {
        self.into_iter().map(|k| (k, ())).collect()
    }

    fn from_equivalent(object: BTreeMap<K, ()>) -> Self {
        object.into_iter().map(|(k, ())| k).collect()
    }
}
