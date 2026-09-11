use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{u63::U63, *};

/// Length-prefixed value. Used to make [`Inline`]s out of arbitrary [`Object`]s.
///
/// If you can guarantee absence of zeroes, see [`zero_terminated::Zt`].
#[pod(no_output, no_parse)]
#[derive(ParseAsInline)]
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

impl<T: SignificantLength> ByteOrd for Lp<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.0.bytes_cmp(&other.0)
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
#[pod(no_copy, no_output, no_parse)]
#[derive(ParseAsInline)]
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

/// Length-prefixed [`String`].
#[pod(no_copy, no_output, no_parse)]
#[derive(ParseAsInline)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LpString(pub String);

impl Display for LpString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&'_ str> for LpString {
    fn from(s: &'_ str) -> Self {
        Self(s.into())
    }
}

impl AsRef<str> for LpString {
    fn as_ref(&self) -> &str {
        self
    }
}

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

#[derive(Debug, ListHashes, Topological, Tagged, ParseAsInline, Clone, PartialEq, Eq, Hash)]
pub struct LpVec<T>(pub Vec<T>);

impl<T: PartialOrd> PartialOrd for LpVec<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.len(), &self.0).partial_cmp(&(other.len(), &other.0))
    }
}

impl<T: Ord> Ord for LpVec<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.len(), &self.0).cmp(&(other.len(), &other.0))
    }
}

impl<T: ByteOrd + InlineOutput> ByteOrd for LpVec<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        (U63::len_of(&self.0), &self.0).bytes_cmp(&(U63::len_of(&other.0), &other.0))
    }
}

impl<T> Deref for LpVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LpVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: InlineOutput> ToOutput for LpVec<T> {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_mangling() {
            self.0.to_output(output);
        }
        if output.is_real() {
            let prefix = U63::len_of(&self.0);
            prefix.to_output(output);
            self.0.to_output(output);
        }
    }
}

impl<T: InlineOutput> InlineOutput for LpVec<T> {}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for LpVec<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let prefix: U63 = input.parse_inline()?;
        Ok(Self(input.parse_vec_n(prefix.as_usize()?)?))
    }
}
