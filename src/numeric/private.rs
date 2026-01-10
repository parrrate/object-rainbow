use std::num::NonZero;

use crate::{enumkind::UsizeTag, *};

pub trait HasOtherSign {
    type OtherSign;
}

type Os<T> = <T as HasOtherSign>::OtherSign;

#[derive(ParseAsInline, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Le<T>(pub T);
#[derive(ParseAsInline, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Be<T>(pub T);

impl<T> From<T> for Be<T> {
    fn from(n: T) -> Self {
        Self(n)
    }
}

impl<T> From<T> for Le<T> {
    fn from(n: T) -> Self {
        Self(n)
    }
}

impl<T> Deref for Le<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for Be<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait AsLe {
    type Le;
    fn construct(self) -> Self::Le;
}

pub trait AsBe {
    type Be;
    fn construct(self) -> Self::Be;
}

macro_rules! signs {
    ($u:ty, $i:ty) => {
        impl HasOtherSign for $u {
            type OtherSign = $i;
        }
        impl HasOtherSign for $i {
            type OtherSign = $u;
        }
    };
}

macro_rules! ae {
    ($n:ty) => {
        impl UsizeTag for $n {
            fn from_usize(n: usize) -> Self {
                n.try_into().expect("discriminant out of range")
            }
            fn to_usize(&self) -> usize {
                (*self).try_into().expect("discriminant out of range")
            }
            fn try_to_usize(&self) -> Option<usize> {
                (*self).try_into().ok()
            }
        }

        impl UsizeTag for NonZero<$n> {
            fn from_usize(n: usize) -> Self {
                Self::new(
                    n.checked_add(1)
                        .expect("discriminant out of range")
                        .try_into()
                        .expect("discriminant out of range"),
                )
                .unwrap()
            }
            fn to_usize(&self) -> usize {
                usize::try_from(self.get())
                    .expect("discriminant out of range")
                    .checked_sub(1)
                    .unwrap()
            }
            fn try_to_usize(&self) -> Option<usize> {
                usize::try_from(self.get()).ok()?.checked_sub(1)
            }
        }

        impl AsLe for $n {
            type Le = Self;
            fn construct(self) -> Self::Le {
                self
            }
        }

        impl AsLe for NonZero<$n> {
            type Le = Self;
            fn construct(self) -> Self::Le {
                self
            }
        }

        impl AsBe for $n {
            type Be = Self;
            fn construct(self) -> Self::Be {
                self
            }
        }

        impl AsBe for NonZero<$n> {
            type Be = Self;
            fn construct(self) -> Self::Be {
                self
            }
        }

        impl ToOutput for NonZero<$n> {
            fn to_output(&self, output: &mut dyn crate::Output) {
                self.get().to_output(output);
            }
        }

        impl<I: ParseInput> Parse<I> for NonZero<$n> {
            fn parse(input: I) -> crate::Result<Self> {
                ParseInline::parse_as_inline(input)
            }
        }

        impl<I: ParseInput> ParseInline<I> for NonZero<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                NonZero::new(input.parse_inline::<$n>()?).ok_or(Error::Zero)
            }
        }

        impl Size for NonZero<$n> {
            const SIZE: usize = <$n as Size>::SIZE;
            type Size = <$n as Size>::Size;
        }

        impl MaybeHasNiche for $n {
            type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
        }

        impl MaybeHasNiche for NonZero<$n> {
            type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
        }

        impl Tagged for $n {}
        impl Tagged for NonZero<$n> {}
        impl ListHashes for $n {}
        impl ListHashes for NonZero<$n> {}
        impl Topological for $n {}
        impl Topological for NonZero<$n> {}

        impl Equivalent<$n> for Option<NonZero<$n>> {
            fn into_equivalent(self) -> $n {
                self.map(NonZero::get).unwrap_or_default()
            }
            fn from_equivalent(object: $n) -> Self {
                NonZero::new(object)
            }
        }

        impl Equivalent<Os<$n>> for $n {
            fn into_equivalent(self) -> Os<$n> {
                self as _
            }
            fn from_equivalent(object: Os<$n>) -> Self {
                object as _
            }
        }

        impl Equivalent<NonZero<Os<$n>>> for NonZero<$n> {
            fn into_equivalent(self) -> NonZero<Os<$n>> {
                NonZero::new(self.get() as _).unwrap()
            }
            fn from_equivalent(object: NonZero<Os<$n>>) -> Self {
                NonZero::new(object.get() as _).unwrap()
            }
        }
    };
}

macro_rules! lebe {
    ($n:ty) => {
        impl UsizeTag for $n {
            fn from_usize(n: usize) -> Self {
                n.try_into().expect("discriminant out of range")
            }
            fn to_usize(&self) -> usize {
                (*self).try_into().expect("discriminant out of range")
            }
            fn try_to_usize(&self) -> Option<usize> {
                (*self).try_into().ok()
            }
        }

        impl UsizeTag for NonZero<$n> {
            fn from_usize(n: usize) -> Self {
                Self::new(
                    n.checked_add(1)
                        .expect("discriminant out of range")
                        .try_into()
                        .expect("discriminant out of range"),
                )
                .unwrap()
            }
            fn to_usize(&self) -> usize {
                usize::try_from(self.get())
                    .expect("discriminant out of range")
                    .checked_sub(1)
                    .unwrap()
            }
            fn try_to_usize(&self) -> Option<usize> {
                usize::try_from(self.get()).ok()?.checked_sub(1)
            }
        }

        impl AsLe for $n {
            type Le = Le<$n>;
            fn construct(self) -> Self::Le {
                Le(self)
            }
        }

        impl AsLe for NonZero<$n> {
            type Le = Le<NonZero<$n>>;
            fn construct(self) -> Self::Le {
                Le(self)
            }
        }

        impl AsBe for $n {
            type Be = Be<$n>;
            fn construct(self) -> Self::Be {
                Be(self)
            }
        }

        impl AsBe for NonZero<$n> {
            type Be = Be<NonZero<$n>>;
            fn construct(self) -> Self::Be {
                Be(self)
            }
        }

        impl ToOutput for Le<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_le_bytes());
            }
        }

        impl ToOutput for Be<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_be_bytes());
            }
        }

        impl ToOutput for Le<NonZero<$n>> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.get().to_le_bytes());
            }
        }

        impl ToOutput for Be<NonZero<$n>> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.get().to_be_bytes());
            }
        }

        impl InlineOutput for Le<$n> {}
        impl InlineOutput for Be<$n> {}

        impl<I: ParseInput> ParseInline<I> for Le<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                Ok(Self(<$n>::from_le_bytes(*input.parse_chunk()?)))
            }
        }

        impl<I: ParseInput> ParseInline<I> for Be<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                Ok(Self(<$n>::from_be_bytes(*input.parse_chunk()?)))
            }
        }

        impl<I: ParseInput> ParseInline<I> for Le<NonZero<$n>> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                NonZero::new(input.parse_inline::<Le<$n>>()?.0)
                    .ok_or(Error::Zero)
                    .map(Le)
            }
        }

        impl<I: ParseInput> ParseInline<I> for Be<NonZero<$n>> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                NonZero::new(input.parse_inline::<Be<$n>>()?.0)
                    .ok_or(Error::Zero)
                    .map(Be)
            }
        }

        impl Size for Le<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
            type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
        }

        impl Size for Be<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
            type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
        }

        impl MaybeHasNiche for Le<$n> {
            type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
        }

        impl MaybeHasNiche for Be<$n> {
            type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
        }

        impl MaybeHasNiche for Le<NonZero<$n>> {
            type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
        }

        impl MaybeHasNiche for Be<NonZero<$n>> {
            type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
        }

        impl Tagged for Le<$n> {}
        impl Tagged for Le<NonZero<$n>> {}
        impl Tagged for Be<$n> {}
        impl Tagged for Be<NonZero<$n>> {}
        impl ListHashes for Le<$n> {}
        impl ListHashes for Le<NonZero<$n>> {}
        impl ListHashes for Be<$n> {}
        impl ListHashes for Be<NonZero<$n>> {}
        impl Topological for Le<$n> {}
        impl Topological for Le<NonZero<$n>> {}
        impl Topological for Be<$n> {}
        impl Topological for Be<NonZero<$n>> {}

        impl Size for Le<NonZero<$n>> {
            const SIZE: usize = <Le<$n> as Size>::SIZE;
            type Size = <Le<$n> as Size>::Size;
        }

        impl Size for Be<NonZero<$n>> {
            const SIZE: usize = <Be<$n> as Size>::SIZE;
            type Size = <Be<$n> as Size>::Size;
        }

        impl Equivalent<Le<$n>> for Option<Le<NonZero<$n>>> {
            fn into_equivalent(self) -> Le<$n> {
                Le(self.map(|object| object.0.get()).unwrap_or_default())
            }
            fn from_equivalent(object: Le<$n>) -> Self {
                NonZero::new(object.0).map(Le)
            }
        }

        impl Equivalent<Be<$n>> for Option<Be<NonZero<$n>>> {
            fn into_equivalent(self) -> Be<$n> {
                Be(self.map(|object| object.0.get()).unwrap_or_default())
            }
            fn from_equivalent(object: Be<$n>) -> Self {
                NonZero::new(object.0).map(Be)
            }
        }

        impl Equivalent<Le<Os<$n>>> for Le<$n> {
            fn into_equivalent(self) -> Le<Os<$n>> {
                Le(self.0 as _)
            }
            fn from_equivalent(object: Le<Os<$n>>) -> Self {
                Le(object.0 as _)
            }
        }

        impl Equivalent<Be<Os<$n>>> for Be<$n> {
            fn into_equivalent(self) -> Be<Os<$n>> {
                Be(self.0 as _)
            }
            fn from_equivalent(object: Be<Os<$n>>) -> Self {
                Be(object.0 as _)
            }
        }

        impl Equivalent<Le<NonZero<Os<$n>>>> for Le<NonZero<$n>> {
            fn into_equivalent(self) -> Le<NonZero<Os<$n>>> {
                Le(NonZero::new(self.0.get() as _).unwrap())
            }
            fn from_equivalent(object: Le<NonZero<Os<$n>>>) -> Self {
                Le(NonZero::new(object.0.get() as _).unwrap())
            }
        }

        impl Equivalent<Be<NonZero<Os<$n>>>> for Be<NonZero<$n>> {
            fn into_equivalent(self) -> Be<NonZero<Os<$n>>> {
                Be(NonZero::new(self.0.get() as _).unwrap())
            }
            fn from_equivalent(object: Be<NonZero<Os<$n>>>) -> Self {
                Be(NonZero::new(object.0.get() as _).unwrap())
            }
        }
    };
}

macro_rules! float {
    ($n:ty) => {
        impl AsLe for $n {
            type Le = Le<$n>;
            fn construct(self) -> Self::Le {
                Le(self)
            }
        }

        impl AsBe for $n {
            type Be = Be<$n>;
            fn construct(self) -> Self::Be {
                Be(self)
            }
        }

        impl ToOutput for Le<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_le_bytes());
            }
        }

        impl ToOutput for Be<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_be_bytes());
            }
        }

        impl InlineOutput for Le<$n> {}
        impl InlineOutput for Be<$n> {}

        impl<I: ParseInput> ParseInline<I> for Le<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                Ok(Self(<$n>::from_le_bytes(*input.parse_chunk()?)))
            }
        }

        impl<I: ParseInput> ParseInline<I> for Be<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                Ok(Self(<$n>::from_be_bytes(*input.parse_chunk()?)))
            }
        }

        impl Size for Le<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
            type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
        }

        impl Size for Be<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
            type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
        }

        impl MaybeHasNiche for Le<$n> {
            type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
        }

        impl MaybeHasNiche for Be<$n> {
            type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
        }

        impl ListHashes for Le<$n> {}
        impl ListHashes for Be<$n> {}
        impl Topological for Le<$n> {}
        impl Topological for Be<$n> {}
        impl Tagged for Le<$n> {}
        impl Tagged for Be<$n> {}
    };
}

signs!(u8, i8);
signs!(u16, i16);
signs!(u32, i32);
signs!(u64, i64);
signs!(u128, i128);

ae!(u8);
ae!(i8);

lebe!(u16);
lebe!(i16);

lebe!(u32);
lebe!(i32);

lebe!(u64);
lebe!(i64);

lebe!(u128);
lebe!(i128);

float!(f32);
float!(f64);

#[test]
fn nonzero() {
    assert_eq!(Option::<super::Le<u8>>::SIZE, 2);
    assert_eq!(Option::<super::Le<NonZero<u8>>>::SIZE, 1);
    assert_eq!(Option::<super::Le<u16>>::SIZE, 3);
    assert_eq!(Option::<super::Le<NonZero<u16>>>::SIZE, 2);
}

impl<T: UsizeTag> UsizeTag for Le<T> {
    fn from_usize(n: usize) -> Self {
        Self(UsizeTag::from_usize(n))
    }

    fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    fn try_to_usize(&self) -> Option<usize> {
        self.0.try_to_usize()
    }
}

impl<T: UsizeTag> UsizeTag for Be<T> {
    fn from_usize(n: usize) -> Self {
        Self(UsizeTag::from_usize(n))
    }

    fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    fn try_to_usize(&self) -> Option<usize> {
        self.0.try_to_usize()
    }
}
