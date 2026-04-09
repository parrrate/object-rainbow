use crate::*;

impl<T: ToOutput> ToOutput for [T] {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: ListPoints> ListPoints for [T] {
    fn list_points(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_points(f);
    }
}

impl<T: Topological> Topological for [T] {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: Tagged> Tagged for [T] {
    const TAGS: Tags = T::TAGS;
}
