use crate::*;

#[derive(Default, Tagged, ListHashes, Topological, Clone)]
pub struct DefaultChain<A, B>(A, B);

impl<A, B: Default> DefaultChain<A, B> {
    pub fn from_first(first: A) -> Self {
        Self(first, Default::default())
    }
}

impl<A: Default, B> DefaultChain<A, B> {
    pub fn from_second(second: B) -> Self {
        Self(Default::default(), second)
    }
}

impl<A: ToOutput + Default + PartialEq, B: ToOutput + Default> ToOutput for DefaultChain<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        if self.0 == A::default() {
            self.1.to_output(output);
        }
    }
}
