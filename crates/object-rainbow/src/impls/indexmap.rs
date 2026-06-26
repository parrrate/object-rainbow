use indexmap::{IndexMap, IndexSet};

use crate::{sequence::PlainCollection, *};

impl<T: InlineOutput> ToOutput for IndexSet<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
    }
}

impl<T: ListHashes> ListHashes for IndexSet<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for IndexSet<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: ParseInline<I> + Eq + std::hash::Hash, I: ParseInput> Parse<I> for IndexSet<T> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_collect()
    }
}

impl<T: Tagged> Tagged for IndexSet<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T> PlainCollection for IndexSet<T> {}

impl<K: InlineOutput, V: InlineOutput> ToOutput for IndexMap<K, V> {
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
    }
}

impl<K: ListHashes, V: ListHashes> ListHashes for IndexMap<K, V> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<K: Topological, V: Topological> Topological for IndexMap<K, V> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<K: ParseInline<I> + Eq + std::hash::Hash, V: ParseInline<I>, I: ParseInput> Parse<I>
    for IndexMap<K, V>
{
    fn parse(input: I) -> crate::Result<Self> {
        input.parse_collect()
    }
}

impl<K: Tagged, V: Tagged> Tagged for IndexMap<K, V> {
    const TAGS: Tags = Tags(&[], &[&K::TAGS, &V::TAGS]);
}

impl<K: Eq + std::hash::Hash> Equivalent<IndexMap<K, ()>> for IndexSet<K> {
    fn into_equivalent(self) -> IndexMap<K, ()> {
        self.into_iter().map(|k| (k, ())).collect()
    }

    fn from_equivalent(object: IndexMap<K, ()>) -> Self {
        object.into_iter().map(|(k, ())| k).collect()
    }
}
