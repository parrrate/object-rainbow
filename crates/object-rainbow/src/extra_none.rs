use crate::{extras::Extras, *};

#[derive(Enum, Debug, Clone, PartialEq, ListHashes, Topological, Tagged)]
#[rainbow(untagged)]
pub enum ExtraNone<T, E = ()> {
    Some(T),
    None(Extras<E>),
}

impl<T, E> ExtraNone<T, E> {
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Some(value) => Some(value),
            Self::None(_) => None,
        }
    }

    pub fn new(extra: Extras<E>, value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Some(value),
            None => Self::None(extra),
        }
    }

    pub fn from_tuple((extra, value): (Extras<E>, Option<T>)) -> Self {
        Self::new(extra, value)
    }
}

pub trait ExtraNoneOutput<E>: Sized {
    fn extra_some_output(&self, output: &mut impl Output);
    fn extra_none_output(extra: &E, output: &mut impl Output);
    fn extra_option_output(option: &ExtraNone<Self, E>, output: &mut impl Output) {
        match option {
            ExtraNone::Some(value) => value.extra_some_output(output),
            ExtraNone::None(extra) => Self::extra_none_output(extra, output),
        }
    }
}

impl<T: OptionOutput, E> ExtraNoneOutput<E> for T {
    fn extra_some_output(&self, output: &mut impl Output) {
        T::to_option_output(Some(self), output);
    }

    fn extra_none_output(_: &E, output: &mut impl Output) {
        T::to_option_output(None, output);
    }

    fn extra_option_output(option: &ExtraNone<Self, E>, output: &mut impl Output) {
        T::to_option_output(option.as_ref(), output);
    }
}

impl<T: ExtraNoneOutput<E>, E> ToOutput for ExtraNone<T, E> {
    fn to_output(&self, output: &mut impl Output) {
        T::extra_option_output(self, output);
    }
}

impl<T: OptionOutput + InlineOutput, E> InlineOutput for ExtraNone<T, E> {}

impl<T: OptionParse<I>, I: PointInput> Parse<I> for ExtraNone<T, I::Extra> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse().map(Self::from_tuple)
    }
}

impl<T: OptionParseInline<I>, I: PointInput> ParseInline<I> for ExtraNone<T, I::Extra> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_inline().map(Self::from_tuple)
    }
}

impl<T: CanonicalExtra<Extra = E>, E: Clone> CanonicalExtra for ExtraNone<T, E> {
    type Extra = T::Extra;

    fn canonical_extra(&self) -> Self::Extra {
        match self {
            Self::Some(value) => value.canonical_extra(),
            Self::None(extra) => extra.canonical_extra(),
        }
    }
}
