use std::ops::{Deref, DerefMut};

use crate::{u63::U63, *};

/// Length-prefixed value. Used to make [`Inline`]s out of arbitrary [`Object`]s.
///
/// If you can guarantee absence of zeroes, see [`zero_terminated::Zt`].
#[derive(ListHashes, Topological, Tagged, ParseAsInline, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lp<T>(pub T);

impl<T> Deref for Lp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Lp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ToOutput> ToOutput for Lp<T> {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_mangling() {
            self.0.to_output(output);
        }
        if output.is_real() {
            let data = self.0.vec();
            let prefix = U63::len_of(&data);
            prefix.to_output(output);
            data.to_output(output);
        }
    }
}

impl<T: ToOutput> InlineOutput for Lp<T> {}

impl<T: Parse<I>, I: ParseInput> ParseInline<I> for Lp<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let prefix: U63 = input.parse_inline()?;
        Ok(Self(input.split_parse(prefix.as_usize()?)?))
    }
}

#[test]
fn prefixed() -> crate::Result<()> {
    let a = Lp(vec![0, 1, 2]);
    let data = a.vec();
    let b = Lp::<Vec<u8>>::parse_slice_refless(&data)?;
    assert_eq!(*a, *b);
    Ok(())
}

/// Length-prefixed [`Vec<u8>`]
#[derive(Debug, Clone, ParseAsInline, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LpBytes(pub Vec<u8>);

impl Deref for LpBytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LpBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ToOutput for LpBytes {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_real() {
            let data = &self.0;
            let prefix = U63::len_of(data);
            prefix.to_output(output);
            data.to_output(output);
        }
    }
}

impl InlineOutput for LpBytes {}

impl<I: ParseInput> ParseInline<I> for LpBytes {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let prefix: U63 = input.parse_inline()?;
        let mut data = vec![0; prefix.as_usize()?];
        input.read(&mut data)?;
        Ok(Self(data))
    }
}

impl Tagged for LpBytes {}
impl ListHashes for LpBytes {}
impl Topological for LpBytes {}

/// Length-prefixed [`String`].
#[derive(Debug, Clone, ParseAsInline, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LpString(pub String);

impl Deref for LpString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LpString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ToOutput for LpString {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_real() {
            let data = &self.0;
            let prefix = U63::len_of(data.as_bytes());
            prefix.to_output(output);
            data.to_output(output);
        }
    }
}

impl InlineOutput for LpString {}

impl<I: ParseInput> ParseInline<I> for LpString {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        String::from_utf8(input.parse_inline::<LpBytes>()?.0)
            .map_err(Error::Utf8)
            .map(Self)
    }
}

impl Tagged for LpString {}
impl ListHashes for LpString {}
impl Topological for LpString {}

#[derive(ListHashes, Topological, Tagged, Clone, PartialEq)]
pub struct LpVec<T>(pub Vec<T>);
