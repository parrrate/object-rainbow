use crate::*;

impl<T: InlineOutput> ToOutput for [T] {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: ListHashes> ListHashes for [T] {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}

impl<T: Topological> Topological for [T] {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.iter_traverse(visitor);
    }
}

impl<T: Tagged> Tagged for [T] {
    const TAGS: Tags = T::TAGS;
}
