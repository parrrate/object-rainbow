use crate::*;

#[derive(ToOutput, ParseAsInline)]
pub struct EnumTag<T, const MAX: usize>(T);

impl<T: ParseInline<I> + Deref<Target: PartialOrd<usize>>, I: ParseInput, const MAX: usize>
    ParseInline<I> for EnumTag<T, MAX>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let n = T::parse_inline(input)?;
        if *n >= MAX {
            Err(Error::DiscriminantOverflow)
        } else {
            Ok(Self(n))
        }
    }
}

pub trait EnumKind: Copy {
    type Tag;
    fn to_tag(self) -> Self::Tag;
    fn from_tag(tag: Self::Tag) -> Self;
}

pub trait Enum {
    type Kind: EnumKind;
    fn kind(&self) -> Self::Kind;
}

pub trait EnumParse<I: ParseInput>: Enum + Parse<I> {
    fn parse_enum(kind: Self::Kind, input: I) -> crate::Result<Self>;
    fn parse_as_enum(mut input: I) -> crate::Result<Self>
    where
        <Self::Kind as EnumKind>::Tag: ParseInline<I>,
    {
        Self::parse_enum(Self::Kind::from_tag(input.parse_inline()?), input)
    }
}

pub trait EnumParseInline<I: ParseInput>: Enum + ParseInline<I> {
    fn parse_inline_enum(kind: Self::Kind, input: &mut I) -> crate::Result<Self>;
    fn parse_as_inline_enum(input: &mut I) -> crate::Result<Self>
    where
        <Self::Kind as EnumKind>::Tag: ParseInline<I>,
    {
        Self::parse_inline_enum(Self::Kind::from_tag(input.parse_inline()?), input)
    }
}
