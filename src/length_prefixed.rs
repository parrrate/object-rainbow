use std::ops::{Add, Deref, DerefMut};

use generic_array::ArrayLength;
use typenum::{Sum, U8, Unsigned, tarr};

use crate::{
    Error, Inline, MaybeHasNiche, NoNiche, Object, Parse, ParseAsInline, ParseInline, ParseInput,
    ReflessInline, ReflessObject, Size, SomeNiche, Tagged, ToOutput, ToOutputExt, Topological,
    ZeroNiche, ZeroNoNiche, numeric::Le,
};

#[derive(Topological, Tagged, Object, Inline, ReflessObject, ReflessInline, ParseAsInline)]
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
    fn to_output(&self, output: &mut dyn crate::Output) {
        let data = self.0.output::<Vec<u8>>();
        let len = data.len();
        let len = len as u64;
        assert_ne!(len, u64::MAX);
        let prefix = Le::<u64>(len);
        prefix.to_output(output);
        data.to_output(output);
    }
}

impl<T: Size> Size for Lp<T>
where
    U8: Add<T::Size, Output: Unsigned>,
{
    type Size = Sum<U8, T::Size>;
}

impl<T: Size> MaybeHasNiche for Lp<T>
where
    U8: Add<T::Size, Output: ArrayLength>,
{
    type MnArray = tarr![SomeNiche<ZeroNiche<U8>>, NoNiche<ZeroNoNiche<T::Size>>];
}

impl<T: Parse<I>, I: ParseInput> ParseInline<I> for Lp<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let prefix: Le<u64> = input.parse_inline()?;
        let len = prefix.0;
        let len = len.try_into().map_err(|_| Error::LenOutOfBounds)?;
        Ok(Self(input.parse_ahead(len)?))
    }
}

#[test]
fn prefixed() {
    let a = Lp(vec![0, 1, 2]);
    let data = a.output::<Vec<u8>>();
    let b = <Lp<Vec<u8>> as ReflessObject>::parse_slice(&data).unwrap();
    assert_eq!(*a, *b);
}

#[derive(ParseAsInline)]
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
    fn to_output(&self, output: &mut dyn crate::Output) {
        let data = &self.0;
        let len = data.len();
        let len = len as u64;
        assert_ne!(len, u64::MAX);
        let prefix = Le::<u64>(len);
        prefix.to_output(output);
        data.to_output(output);
    }
}

impl<I: ParseInput> ParseInline<I> for LpBytes {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let prefix: Le<u64> = input.parse_inline()?;
        let len = prefix.0;
        let len = len.try_into().map_err(|_| Error::LenOutOfBounds)?;
        Ok(Self(input.parse_n(len)?.into()))
    }
}

impl Topological for LpBytes {}
impl Tagged for LpBytes {}
impl Object for LpBytes {}
impl Inline for LpBytes {}
impl ReflessObject for LpBytes {}
impl ReflessInline for LpBytes {}

#[derive(Debug, Clone, ParseAsInline, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    fn to_output(&self, output: &mut dyn crate::Output) {
        let data = self.0.as_bytes();
        let len = data.len();
        let len = len as u64;
        assert_ne!(len, u64::MAX);
        let prefix = Le::<u64>(len);
        prefix.to_output(output);
        data.to_output(output);
    }
}

impl<I: ParseInput> ParseInline<I> for LpString {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        String::from_utf8(input.parse_inline::<LpBytes>()?.0)
            .map_err(Error::Utf8)
            .map(Self)
    }
}

impl Topological for LpString {}
impl Tagged for LpString {}
impl Object for LpString {}
impl Inline for LpString {}
impl ReflessObject for LpString {}
impl ReflessInline for LpString {}
