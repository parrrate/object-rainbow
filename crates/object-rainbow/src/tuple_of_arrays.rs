use crate::ToOutput;

pub struct TupleOfArrays<A, B>(A, B);

impl<A: ToOutput, B: ToOutput> ToOutput for TupleOfArrays<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}
