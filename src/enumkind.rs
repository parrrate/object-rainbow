use crate::*;

pub trait UsizeTag: Sized {
    fn from_usize(n: usize) -> Self;
    fn to_usize(&self) -> usize;
}

#[derive(ToOutput, Topological, Tagged, ParseAsInline, Size)]
pub struct EnumTag<T, const MAX: usize>(T);

impl<T: Deref + From<T::Target>, const MAX: usize> EnumTag<T, MAX>
where
    T::Target: UsizeTag,
{
    pub fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    pub fn from_const<const N: usize>() -> Self {
        assert!(N < MAX);
        Self(T::from(UsizeTag::from_usize(N)))
    }
}

impl<T: ParseInline<I> + Deref<Target: Copy + TryInto<usize>>, I: ParseInput, const MAX: usize>
    ParseInline<I> for EnumTag<T, MAX>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let n_raw = T::parse_inline(input)?;
        let n: Option<usize> = (*n_raw).try_into().ok();
        if let Some(n) = n {
            if n < MAX {
                return Ok(Self(n_raw));
            }
        }
        Err(Error::DiscriminantOverflow)
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
