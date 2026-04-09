use std::num::NonZero;

use crate::{enumkind::UsizeTag, *};

pub trait HasOtherSign {
    type OtherSign;
}

type Os<T> = <T as HasOtherSign>::OtherSign;

#[derive(ParseAsInline)]
pub struct Ae<T>(T);
#[derive(ParseAsInline)]
pub struct Le<T>(T);
#[derive(ParseAsInline)]
pub struct Be<T>(T);
#[derive(ParseAsInline)]
pub struct Nz<T: NonZeroable>(T::Nz);

pub trait NonZeroable {
    type Nz: Send + Sync;
    fn to_nz(&self) -> Option<Self::Nz>;
    fn from_nz(nz: &Self::Nz) -> Self;
}

impl<T> From<T> for Ae<T> {
    fn from(n: T) -> Self {
        Self(n)
    }
}

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

impl<T> Deref for Ae<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
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
}

pub trait AsBe {
    type Be;
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
            type Le = Ae<$n>;
        }

        impl AsLe for NonZero<$n> {
            type Le = Nz<Ae<$n>>;
        }

        impl AsBe for $n {
            type Be = Ae<$n>;
        }

        impl AsBe for NonZero<$n> {
            type Be = Nz<Ae<$n>>;
        }

        impl From<NonZero<$n>> for Nz<Ae<$n>> {
            fn from(nz: NonZero<$n>) -> Self {
                Self(nz)
            }
        }

        impl NonZeroable for Ae<$n> {
            type Nz = NonZero<$n>;
            fn to_nz(&self) -> Option<Self::Nz> {
                NonZero::new(self.0)
            }
            fn from_nz(nz: &Self::Nz) -> Self {
                Self(nz.get())
            }
        }

        impl ToOutput for Ae<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_le_bytes());
            }
        }

        impl<I: ParseInput> ParseInline<I> for Ae<$n> {
            fn parse_inline(input: &mut I) -> crate::Result<Self> {
                Ok(Self(<$n>::from_le_bytes(*input.parse_chunk()?)))
            }
        }

        impl Size for Ae<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
            type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
        }

        impl MaybeHasNiche for Ae<$n> {
            type MnArray = NoNiche<<Self as Size>::Size>;
        }

        impl Topological for Ae<$n> {
            fn accept_points(&self, _: &mut impl PointVisitor) {}
        }

        impl Tagged for Ae<$n> {}
        impl Tagged for Ae<NonZero<$n>> {}
        impl ReflessObject for Ae<$n> {}
        impl ReflessInline for Ae<$n> {}
        impl Object for Ae<$n> {}
        impl Inline for Ae<$n> {}

        impl Equivalent<Ae<$n>> for Option<Ae<NonZero<$n>>> {
            fn into_equivalent(self) -> Ae<$n> {
                Ae(self.map(|object| object.0.get()).unwrap_or_default())
            }
            fn from_equivalent(object: Ae<$n>) -> Self {
                NonZero::new(object.0).map(Ae)
            }
        }

        impl Equivalent<Ae<Os<$n>>> for Ae<$n> {
            fn into_equivalent(self) -> Ae<Os<$n>> {
                Ae(self.0 as _)
            }
            fn from_equivalent(object: Ae<Os<$n>>) -> Self {
                Ae(object.0 as _)
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
        }

        impl AsLe for NonZero<$n> {
            type Le = Nz<Le<$n>>;
        }

        impl AsBe for $n {
            type Be = Be<$n>;
        }

        impl AsBe for NonZero<$n> {
            type Be = Nz<Be<$n>>;
        }

        impl NonZeroable for Le<$n> {
            type Nz = NonZero<$n>;
            fn to_nz(&self) -> Option<Self::Nz> {
                NonZero::new(self.0)
            }
            fn from_nz(nz: &Self::Nz) -> Self {
                Self(nz.get())
            }
        }

        impl NonZeroable for Be<$n> {
            type Nz = NonZero<$n>;
            fn to_nz(&self) -> Option<Self::Nz> {
                NonZero::new(self.0)
            }
            fn from_nz(nz: &Self::Nz) -> Self {
                Self(nz.get())
            }
        }

        impl From<NonZero<$n>> for Nz<Le<$n>> {
            fn from(nz: NonZero<$n>) -> Self {
                Self(nz)
            }
        }

        impl From<NonZero<$n>> for Nz<Be<$n>> {
            fn from(nz: NonZero<$n>) -> Self {
                Self(nz)
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
            type MnArray = NoNiche<<Self as Size>::Size>;
        }

        impl MaybeHasNiche for Be<$n> {
            type MnArray = NoNiche<<Self as Size>::Size>;
        }

        impl Topological for Le<$n> {
            fn accept_points(&self, _: &mut impl PointVisitor) {}
        }

        impl Topological for Be<$n> {
            fn accept_points(&self, _: &mut impl PointVisitor) {}
        }

        impl Tagged for Le<$n> {}
        impl Tagged for Be<$n> {}
        impl ReflessObject for Le<$n> {}
        impl ReflessObject for Be<$n> {}
        impl ReflessInline for Le<$n> {}
        impl ReflessInline for Be<$n> {}
        impl Object for Le<$n> {}
        impl Object for Be<$n> {}
        impl Inline for Le<$n> {}
        impl Inline for Be<$n> {}

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

impl<T: NonZeroable> Deref for Nz<T> {
    type Target = T::Nz;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: NonZeroable + ToOutput> ToOutput for Nz<T> {
    fn to_output(&self, output: &mut dyn Output) {
        T::from_nz(&self.0).to_output(output);
    }
}

impl<T: NonZeroable + ParseInline<I>, I: ParseInput> ParseInline<I> for Nz<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self(T::parse_inline(input)?.to_nz().ok_or(Error::Zero)?))
    }
}

impl<T: NonZeroable + Size> Size for Nz<T> {
    type Size = T::Size;
}

impl<T: NonZeroable + Size> MaybeHasNiche for Nz<T> {
    type MnArray = SomeNiche<ZeroNiche<T::Size>>;
}

impl<T: NonZeroable> Topological for Nz<T> {
    fn accept_points(&self, _: &mut impl PointVisitor) {}
}

impl<T: NonZeroable> Tagged for Nz<T> {}
impl<T: NonZeroable + ReflessInline> ReflessObject for Nz<T> {}
impl<T: NonZeroable + ReflessInline> ReflessInline for Nz<T> {}
impl<T: NonZeroable + Inline> Object for Nz<T> {}
impl<T: NonZeroable + Inline> Inline for Nz<T> {}

#[test]
fn nonzero() {
    assert_eq!(Option::<Ae<u8>>::SIZE, 2);
    assert_eq!(Option::<Nz<Ae<u8>>>::SIZE, 1);
    assert_eq!(Option::<Le<u16>>::SIZE, 3);
    assert_eq!(Option::<Nz<Le<u16>>>::SIZE, 2);
}
