use crate::*;

impl<T: ToOutput> ToOutput for [T] {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: Topological<E>, E: 'static> Topological<E> for [T] {
    fn accept_points(&self, visitor: &mut impl PointVisitor<E>) {
        self.iter_accept_points(visitor);
    }
}

impl<T: Tagged> Tagged for [T] {
    const TAGS: Tags = T::TAGS;
}
