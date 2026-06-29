use std::num::NonZero;

use object_rainbow::u63::U63;

use crate::*;

#[derive(
    Debug,
    Enum,
    ToOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Clone,
    PartialEq,
)]
#[enumtag("char")]
#[niche(tag)]
pub enum NumericSchema {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    U256(Infallible),
    I256(Infallible),
    NzU8,
    NzU16,
    NzU32,
    NzU64,
    NzU128,
    NzU256(Infallible),
    F8(Infallible),
    F16(Infallible),
    F32,
    F64,
    F128(Infallible),
    F256(Infallible),
    OpaqueChar,
    OpaqueBool,
    LpU63,
}

impl InlineOutput for NumericSchema {}
impl Tagged for NumericSchema {}

impl AbstractSchema for NumericSchema {
    fn niche(&self) -> SchemaNiche {
        match self.clone() {
            Self::U8 | Self::I8 => SchemaNiche::ZeroNoNiche(1),
            Self::U16 | Self::I16 => SchemaNiche::ZeroNoNiche(2),
            Self::U32 | Self::I32 => SchemaNiche::ZeroNoNiche(4),
            Self::U64 | Self::I64 => SchemaNiche::ZeroNoNiche(8),
            Self::U128 | Self::I128 => SchemaNiche::ZeroNoNiche(16),
            Self::NzU8 => SchemaNiche::Zeroes(1),
            Self::NzU16 => SchemaNiche::Zeroes(2),
            Self::NzU32 => SchemaNiche::Zeroes(4),
            Self::NzU64 => SchemaNiche::Zeroes(8),
            Self::NzU128 => SchemaNiche::Zeroes(16),
            Self::F32 => SchemaNiche::ZeroNoNiche(4),
            Self::F64 => SchemaNiche::ZeroNoNiche(8),
            Self::OpaqueChar => SchemaNiche::Cut,
            Self::OpaqueBool => SchemaNiche::ZeroNoNiche(1),
            Self::LpU63 => SchemaNiche::Cut,
        }
    }
}

impl DefaultSchema<NumericValue> for NumericSchema {
    fn default_value(&self) -> Option<NumericValue> {
        Some(match self.clone() {
            Self::U8 => NumericValue::U8(Default::default()),
            Self::I8 => NumericValue::I8(Default::default()),
            Self::U16 => NumericValue::U16(Default::default()),
            Self::I16 => NumericValue::I16(Default::default()),
            Self::U32 => NumericValue::U32(Default::default()),
            Self::I32 => NumericValue::I32(Default::default()),
            Self::U64 => NumericValue::U64(Default::default()),
            Self::I64 => NumericValue::I64(Default::default()),
            Self::U128 => NumericValue::U128(Default::default()),
            Self::I128 => NumericValue::I128(Default::default()),
            Self::NzU8 => NumericValue::NzU8(NonZero::new(1).expect("1 != 0")),
            Self::NzU16 => NumericValue::NzU16(NonZero::new(1).expect("1 != 0")),
            Self::NzU32 => NumericValue::NzU32(NonZero::new(1).expect("1 != 0")),
            Self::NzU64 => NumericValue::NzU64(NonZero::new(1).expect("1 != 0")),
            Self::NzU128 => NumericValue::NzU128(NonZero::new(1).expect("1 != 0")),
            Self::F32 => NumericValue::F32(Default::default()),
            Self::F64 => NumericValue::F64(Default::default()),
            Self::OpaqueChar => NumericValue::OpaqueChar(Default::default()),
            Self::OpaqueBool => NumericValue::OpaqueBool(Default::default()),
            Self::LpU63 => NumericValue::LpU63(Default::default()),
        })
    }
}

impl DefaultIsMin for NumericSchema {
    fn default_is_min(&self) -> bool {
        true
    }
}

impl SizeSchema for NumericSchema {
    fn size(&self) -> Option<u64> {
        match self.clone() {
            Self::U8 | Self::I8 | Self::NzU8 => Some(1),
            Self::U16 | Self::I16 | Self::NzU16 => Some(2),
            Self::U32 | Self::I32 | Self::NzU32 | Self::F32 => Some(4),
            Self::U64 | Self::I64 | Self::NzU64 | Self::F64 => Some(8),
            Self::U128 | Self::I128 | Self::NzU128 => Some(16),
            Self::OpaqueChar => None,
            Self::OpaqueBool => Some(1),
            Self::LpU63 => None,
        }
    }
}

impl From<NumericSchema> for InlineSchema {
    fn from(schema: NumericSchema) -> Self {
        Self::Numeric(schema)
    }
}

#[derive(Debug, ToOutput, ListHashes, Topological, ParseAsInline, PartialEq)]
#[rainbow(untagged)]
pub enum NumericValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(u64),
    U128(u128),
    I128(i128),
    NzU8(NonZero<u8>),
    NzU16(NonZero<u16>),
    NzU32(NonZero<u32>),
    NzU64(NonZero<u64>),
    NzU128(NonZero<u128>),
    F32(f32),
    F64(f64),
    OpaqueChar(char),
    OpaqueBool(bool),
    LpU63(U63),
}

impl InlineOutput for NumericValue {}

impl NumericValue {
    pub fn index(&self) -> Option<usize> {
        match self {
            Self::U8(x) => Some(*x as _),
            Self::I8(x) => Some(*x as _),
            Self::U16(x) => Some(*x as _),
            Self::I16(x) => Some(*x as _),
            Self::U32(x) => Some(*x as _),
            Self::I32(x) => Some(*x as _),
            Self::U64(x) => (*x).try_into().ok(),
            Self::I64(x) => (*x).try_into().ok(),
            Self::U128(x) => (*x).try_into().ok(),
            Self::I128(x) => (*x).try_into().ok(),
            Self::NzU8(x) => Some((x.get() - 1) as _),
            Self::NzU16(x) => Some((x.get() - 1) as _),
            Self::NzU32(x) => Some((x.get() - 1) as _),
            Self::NzU64(x) => (x.get() - 1).try_into().ok(),
            Self::NzU128(x) => (x.get() - 1).try_into().ok(),
            Self::F32(_) => None,
            Self::F64(_) => None,
            Self::OpaqueChar(x) => Some(*x as _),
            Self::OpaqueBool(x) => Some(*x as _),
            Self::LpU63(x) => x.as_usize().ok(),
        }
    }
}
