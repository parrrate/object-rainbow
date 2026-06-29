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
