use crate::*;

impl<T: ToOutput> ToOutput for [T] {
    fn to_output(&self, output: &mut dyn Output) {
        self.iter_to_output(output);
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
