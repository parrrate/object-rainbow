use crate::*;

#[derive(ParseAsInline)]
pub struct Ae<T>(T);
#[derive(ParseAsInline)]
pub struct Le<T>(T);
#[derive(ParseAsInline)]
pub struct Be<T>(T);

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

macro_rules! ae {
    ($n:ty) => {
        impl AsLe for $n {
            type Le = Ae<$n>;
        }

        impl AsBe for $n {
            type Be = Ae<$n>;
        }

        impl ToOutput for Ae<$n> {
            fn to_output(&self, output: &mut dyn Output) {
                output.write(&self.0.to_le_bytes());
            }
        }

        impl ParseInline<ReflessInput<'_>> for Ae<$n> {
            fn parse_inline(input: &mut ReflessInput<'_>) -> crate::Result<Self> {
                Ok(Self(<$n>::from_le_bytes(*input.parse_chunk()?)))
            }
        }

        impl Size for Ae<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
        }

        impl Tagged for Ae<$n> {}
        impl ReflessObject for Ae<$n> {}
        impl ReflessInline for Ae<$n> {}
    };
}

macro_rules! lebe {
    ($n:ty) => {
        impl AsLe for $n {
            type Le = Le<$n>;
        }

        impl AsBe for $n {
            type Be = Be<$n>;
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

        impl ParseInline<ReflessInput<'_>> for Le<$n> {
            fn parse_inline(input: &mut ReflessInput<'_>) -> crate::Result<Self> {
                Ok(Self(<$n>::from_le_bytes(*input.parse_chunk()?)))
            }
        }

        impl ParseInline<ReflessInput<'_>> for Be<$n> {
            fn parse_inline(input: &mut ReflessInput<'_>) -> crate::Result<Self> {
                Ok(Self(<$n>::from_be_bytes(*input.parse_chunk()?)))
            }
        }

        impl Size for Le<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
        }

        impl Size for Be<$n> {
            const SIZE: usize = std::mem::size_of::<$n>();
        }

        impl Tagged for Le<$n> {}
        impl Tagged for Be<$n> {}
        impl ReflessObject for Le<$n> {}
        impl ReflessObject for Be<$n> {}
        impl ReflessInline for Le<$n> {}
        impl ReflessInline for Be<$n> {}
    };
}

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
