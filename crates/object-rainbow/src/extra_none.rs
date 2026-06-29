use crate::*;

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
