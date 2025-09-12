use crate::*;

#[derive(ToOutput, ParseAsInline, Topological)]
pub struct EnumTag<T, const MAX: usize>(T);

impl<T: Deref, const MAX: usize> Deref for EnumTag<T, MAX> {
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Deref + From<T::Target>, const MAX: usize> EnumTag<T, MAX>
where
    T::Target: TryFrom<usize>,
{
    pub fn from_const<const N: usize>() -> Self {
        assert!(N < MAX);
        match N.try_into() {
            Ok(n) => Self(T::from(n)),
            Err(_) => panic!("cannot convert"),
        }
    }
}

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
    fn enum_parse(kind: Self::Kind, input: I) -> crate::Result<Self>;
    fn parse_as_enum(mut input: I) -> crate::Result<Self>
    where
        <Self::Kind as EnumKind>::Tag: ParseInline<I>,
    {
        Self::enum_parse(Self::Kind::from_tag(input.parse_inline()?), input)
    }
}

pub trait EnumParseInline<I: ParseInput>: Enum + ParseInline<I> {
    fn enum_parse_inline(kind: Self::Kind, input: &mut I) -> crate::Result<Self>;
    fn parse_as_inline_enum(input: &mut I) -> crate::Result<Self>
    where
        <Self::Kind as EnumKind>::Tag: ParseInline<I>,
    {
        Self::enum_parse_inline(Self::Kind::from_tag(input.parse_inline()?), input)
    }
}
