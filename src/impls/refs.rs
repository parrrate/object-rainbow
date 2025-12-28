use crate::{ToOutput, Topological};

impl<T: ToOutput> ToOutput for &T {
    fn to_output(&self, output: &mut dyn crate::Output) {
        (**self).to_output(output);
    }
}

impl<T: Topological> Topological for &T {
    fn accept_points(&self, visitor: &mut impl crate::PointVisitor) {
        (**self).accept_points(visitor);
    }
}
