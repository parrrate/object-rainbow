use crate::ToOutput;

pub struct DefaultChain<A, B>(A, B);

impl<A: ToOutput + Default + PartialEq, B: ToOutput + Default> ToOutput for DefaultChain<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        if self.0 == A::default() {
            self.1.to_output(output);
        }
    }
}
