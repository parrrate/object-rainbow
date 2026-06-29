use crate::*;

#[derive(Debug)]
pub enum ExtraNone<T, E = ()> {
    Some(T),
    None(E),
}

impl<T, E> ExtraNone<T, E> {
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Some(value) => Some(value),
            Self::None(_) => None,
        }
    }
}

impl<T: OptionOutput, E> ToOutput for ExtraNone<T, E> {
    fn to_output(&self, output: &mut impl crate::Output) {
        T::to_option_output(self.as_ref(), output);
    }
}

impl<T: OptionOutput + InlineOutput, E> InlineOutput for ExtraNone<T, E> {}

impl<T: Tagged, E> Tagged for ExtraNone<T, E> {
    const TAGS: Tags = T::TAGS;
}

impl<T: ListHashes, E> ListHashes for ExtraNone<T, E> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.as_ref().list_hashes(f);
    }
}

impl<T: Topological, E> Topological for ExtraNone<T, E> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.as_ref().traverse(visitor);
    }
}
