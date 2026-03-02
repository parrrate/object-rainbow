use crate::*;

pub trait UsizeTag: Sized {
    /// Used by [`EnumTag::from_const`].
    ///
    /// ## Panics
    ///
    /// Panics on out-of-bounds.
    fn from_usize(n: usize) -> Self;
    fn to_usize(&self) -> usize;
    fn try_to_usize(&self) -> Option<usize>;
}

impl UsizeTag for bool {
    fn from_usize(n: usize) -> Self {
        match n {
            0 => false,
            1 => true,
            _ => panic!("out of bounds"),
        }
    }

    fn to_usize(&self) -> usize {
        *self as _
    }

    fn try_to_usize(&self) -> Option<usize> {
        Some(self.to_usize())
    }
}

#[derive(
    ToOutput, InlineOutput, ListHashes, Topological, Tagged, ParseAsInline, Size, MaybeHasNiche,
)]
pub struct EnumTag<T, const MAX: usize>(T);

impl<T: UsizeTag, const MAX: usize> UsizeTag for EnumTag<T, MAX> {
    fn from_usize(n: usize) -> Self {
        assert!(n < MAX);
        Self(UsizeTag::from_usize(n))
    }

    fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    fn try_to_usize(&self) -> Option<usize> {
        self.0.try_to_usize()
    }
}

impl<T: UsizeTag, const MAX: usize> EnumTag<T, MAX> {
    pub fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    pub fn from_const<const N: usize>() -> Self {
        assert!(N < MAX);
        Self(UsizeTag::from_usize(N))
    }
}

impl<T: ParseInline<I> + UsizeTag, I: ParseInput, const MAX: usize> ParseInline<I>
    for EnumTag<T, MAX>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let n_raw = T::parse_inline(input)?;
        let n: Option<usize> = n_raw.try_to_usize();
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
