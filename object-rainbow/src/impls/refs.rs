use crate::*;

impl<T: ?Sized + ToOutput> ToOutput for &T {
    fn to_output(&self, output: &mut impl Output) {
        (**self).to_output(output);
    }
}

impl<T: ?Sized + InlineOutput> InlineOutput for &T {}

impl<T: ?Sized + ListHashes> ListHashes for &T {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        (**self).list_hashes(f);
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn point_count(&self) -> usize {
        (**self).point_count()
    }
}

impl<T: ?Sized + Tagged> Tagged for &T {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}

impl<T: ?Sized + Topological> Topological for &T {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        (**self).traverse(visitor);
    }
}
