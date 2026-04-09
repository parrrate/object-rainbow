use crate::*;

impl<T: ToOutput> ToOutput for &T {
    fn to_output(&self, output: &mut dyn Output) {
        (**self).to_output(output);
    }
}

impl<T: InlineOutput> InlineOutput for &T {}

impl<T: ListPoints> ListPoints for &T {
    fn list_points(&self, f: &mut impl FnMut(Hash)) {
        (**self).list_points(f);
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn point_count(&self) -> usize {
        (**self).point_count()
    }
}

impl<T: Tagged> Tagged for &T {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}

impl<T: Topological> Topological for &T {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        (**self).accept_points(visitor);
    }
}
