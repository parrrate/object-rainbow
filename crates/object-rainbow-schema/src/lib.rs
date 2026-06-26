use std::{convert::Infallible, num::NonZero, sync::Arc};

use object_rainbow::{
    Enum, Inline, InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseAsInline,
    ParseInline, PointInput, ReflessInline, Tagged, ToOutput, Topological, Traversible,
    assert_impl,
    extras::Extras,
    length_prefixed::LpVec,
    map_extra::MappedExtra,
    tuple_extra::{Extra0, Extra1},
    tuple_of_arrays::try_divide,
    u63::U63,
    zero_terminated::Zt,
};

#[cfg(feature = "_collections")]
use self::collections::{CollectionSchema, CollectionValue};
#[cfg(feature = "point")]
use self::point::{PointSchema, ValuePoint};

#[cfg(feature = "_collections")]
pub mod collections;
#[cfg(feature = "point")]
pub mod point;

pub trait AbstractSchema: ReflessInline + Traversible {
    fn niche(&self) -> SchemaNiche;
}

pub trait OptionSchema: AbstractSchema {
    fn option(self: Arc<Self>) -> Self;
}

pub trait AbstractValue: ToOutput {
    type Schema: AbstractSchema;
    fn schema(&self) -> Self::Schema;
}

impl AbstractSchema for Infallible {
    fn niche(&self) -> SchemaNiche {
        match *self {}
    }
}

impl DefaultIsMin for Infallible {
    fn default_is_min(&self) -> bool {
        match *self {}
    }
}

pub trait DefaultIsMin {
    fn default_is_min(&self) -> bool;
}

pub trait DefaultSchema<T: AbstractValue<Schema = Self>>: AbstractSchema {
    fn default_value(&self) -> Option<T>;
}

pub trait SizeSchema {
    fn size(&self) -> Option<u64>;
}

impl SizeSchema for Infallible {
    fn size(&self) -> Option<u64> {
        match *self {}
    }
}

pub trait ItemSizeSchema {
    fn item_size(&self) -> Option<u64>;
}

impl ItemSizeSchema for Infallible {
    fn item_size(&self) -> Option<u64> {
        match *self {}
    }
}

pub trait AbstractCollection {
    fn items(&self) -> Vec<Arc<InlineValue>>;
}

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

impl AbstractValue for NumericValue {
    type Schema = NumericSchema;

    fn schema(&self) -> Self::Schema {
        match self {
            Self::U8(_) => NumericSchema::U8,
            Self::I8(_) => NumericSchema::I8,
            Self::U16(_) => NumericSchema::U16,
            Self::I16(_) => NumericSchema::I16,
            Self::U32(_) => NumericSchema::U32,
            Self::I32(_) => NumericSchema::I32,
            Self::U64(_) => NumericSchema::U64,
            Self::I64(_) => NumericSchema::I64,
            Self::U128(_) => NumericSchema::U128,
            Self::I128(_) => NumericSchema::I128,
            Self::NzU8(_) => NumericSchema::NzU8,
            Self::NzU16(_) => NumericSchema::NzU16,
            Self::NzU32(_) => NumericSchema::NzU32,
            Self::NzU64(_) => NumericSchema::NzU64,
            Self::NzU128(_) => NumericSchema::NzU128,
            Self::F32(_) => NumericSchema::F32,
            Self::F64(_) => NumericSchema::F64,
            Self::OpaqueChar(_) => NumericSchema::OpaqueChar,
            Self::OpaqueBool(_) => NumericSchema::OpaqueBool,
            Self::LpU63(_) => NumericSchema::LpU63,
        }
    }
}

impl InlineOutput for NumericValue {}
impl Tagged for NumericValue {}

impl<I: PointInput<Extra = NumericSchema>> ParseInline<I> for NumericValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(match input.extra().clone() {
            NumericSchema::U8 => Self::U8(input.parse_inline()?),
            NumericSchema::I8 => Self::I8(input.parse_inline()?),
            NumericSchema::U16 => Self::U16(input.parse_inline()?),
            NumericSchema::I16 => Self::I16(input.parse_inline()?),
            NumericSchema::U32 => Self::U32(input.parse_inline()?),
            NumericSchema::I32 => Self::I32(input.parse_inline()?),
            NumericSchema::U64 => Self::U64(input.parse_inline()?),
            NumericSchema::I64 => Self::I64(input.parse_inline()?),
            NumericSchema::U128 => Self::U128(input.parse_inline()?),
            NumericSchema::I128 => Self::I128(input.parse_inline()?),
            NumericSchema::NzU8 => Self::NzU8(input.parse_inline()?),
            NumericSchema::NzU16 => Self::NzU16(input.parse_inline()?),
            NumericSchema::NzU32 => Self::NzU32(input.parse_inline()?),
            NumericSchema::NzU64 => Self::NzU64(input.parse_inline()?),
            NumericSchema::NzU128 => Self::NzU128(input.parse_inline()?),
            NumericSchema::F32 => Self::F32(input.parse_inline()?),
            NumericSchema::F64 => Self::F64(input.parse_inline()?),
            NumericSchema::OpaqueChar => Self::OpaqueChar(input.parse_inline()?),
            NumericSchema::OpaqueBool => Self::OpaqueBool(input.parse_inline()?),
            NumericSchema::LpU63 => Self::LpU63(input.parse_inline()?),
        })
    }
}

impl From<NumericValue> for InlineValue {
    fn from(value: NumericValue) -> Self {
        Self::Numeric(value)
    }
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Tagged,
    Clone,
    PartialEq,
)]
pub struct ArraySchema {
    pub len: u64,
    pub schema: Arc<InlineSchema>,
}

impl AbstractSchema for ArraySchema {
    fn niche(&self) -> SchemaNiche {
        self.schema.niche().repeat(self.len)
    }
}

impl DefaultSchema<ValueArray> for ArraySchema {
    fn default_value(&self) -> Option<ValueArray> {
        Some(ValueArray {
            items: std::iter::repeat_n(self.schema.default_value().map(Arc::new), self.len as _)
                .collect::<Option<_>>()?,
            schema: self.schema.clone(),
        })
    }
}

impl DefaultIsMin for ArraySchema {
    fn default_is_min(&self) -> bool {
        self.len == 0 || self.schema.default_is_min()
    }
}

impl From<ArraySchema> for InlineSchema {
    fn from(schema: ArraySchema) -> Self {
        Self::Array(schema)
    }
}

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
#[parse(unchecked)]
pub enum InlineSchema {
    Never,
    Unit,
    Option(Arc<Self>),
    Point(
        #[cfg(feature = "point")] PointSchema,
        #[cfg(not(feature = "point"))] Infallible,
    ),
    Nt(Arc<Self>),
    Concat(Arc<Self>, Arc<Self>),
    Array(ArraySchema),
    Numeric(NumericSchema),
    Enum(EnumSchema<Self>),
    Collection(
        #[cfg(feature = "_collections")] CollectionSchema,
        #[cfg(not(feature = "_collections"))] Infallible,
    ),
    Zt(Arc<TailSchema>),
}

impl InlineOutput for InlineSchema {}
impl Tagged for InlineSchema {}

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
#[parse(unchecked)]
pub enum TailSchema {
    Cut,
    Option(Arc<Self>),
    Sequence(Arc<InlineSchema>),
    Concat(Arc<InlineSchema>, Arc<Self>),
    ToA(Arc<Self>, Arc<Self>),
    Enum(EnumSchema<Self>),
    Bytes,
    String,
}

impl InlineOutput for TailSchema {}
impl Tagged for TailSchema {}

#[derive(Debug, ListHashes, Topological, Tagged, PartialEq)]
#[rainbow(untagged)]
pub enum ValueOption<T: AbstractValue> {
    None(Arc<T::Schema>),
    Some(Arc<T>),
}

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct ValueNt {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl AbstractCollection for ValueNt {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items.clone()
    }
}

impl ToOutput for ValueNt {
    fn to_output(&self, output: &mut impl Output) {
        for item in &self.items {
            ValueOption::Some(item.clone()).to_output(output);
        }
        ValueOption::<InlineValue>::None(self.schema.clone()).to_output(output);
    }
}

impl InlineOutput for ValueNt {}
impl Tagged for ValueNt {}

impl AbstractValue for ValueNt {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Nt(self.schema.clone())
    }
}

impl<I: PointInput<Extra = Arc<InlineSchema>>> ParseInline<I> for ValueNt
where
    ValueOption<InlineValue>: ParseInline<I>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let mut items = Vec::new();
        let schema = loop {
            match input.parse_inline()? {
                ValueOption::Some(item) => items.push(item),
                ValueOption::None(schema) => break schema,
            }
        };
        Ok(Self { items, schema })
    }
}

#[derive(Debug, ParseAsInline, ListHashes, Topological, PartialEq)]
pub struct ValueArray {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl AbstractCollection for ValueArray {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items.clone()
    }
}

impl ToOutput for ValueArray {
    fn to_output(&self, output: &mut impl Output) {
        self.items.to_output(output);
    }
}

impl InlineOutput for ValueArray {}
impl Tagged for ValueArray {}

impl AbstractValue for ValueArray {
    type Schema = ArraySchema;

    fn schema(&self) -> Self::Schema {
        ArraySchema {
            len: self.items.len() as _,
            schema: self.schema.clone(),
        }
    }
}

impl<I: PointInput<Extra = ArraySchema>> ParseInline<I> for ValueArray
where
    InlineValue: ParseInline<I::WithExtra<Arc<InlineSchema>>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let ArraySchema { len, schema } = input.extra().clone();
        let mut items = Vec::new();
        for _ in 0..len {
            items.push(input.parse_inline_extra(schema.clone())?);
        }
        Ok(Self { items, schema })
    }
}

impl From<ValueArray> for InlineValue {
    fn from(value: ValueArray) -> Self {
        Self::Array(value)
    }
}

#[derive(Debug, ToOutput, ListHashes, Topological, Parse, PartialEq)]
pub struct ValueSequence {
    pub schema: Extras<Arc<InlineSchema>>,
    pub items: Vec<Arc<InlineValue>>,
}

impl AbstractCollection for ValueSequence {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.items.clone()
    }
}

impl Tagged for ValueSequence {}

impl AbstractValue for ValueSequence {
    type Schema = TailSchema;

    fn schema(&self) -> Self::Schema {
        TailSchema::Sequence(self.schema.0.clone())
    }
}

#[derive(Debug, ListHashes, Topological, PartialEq)]
pub struct ValueToA(
    pub MappedExtra<Arc<TailValue>, Extra0>,
    pub MappedExtra<Arc<TailValue>, Extra1>,
);

impl ToOutput for ValueToA {
    fn to_output(&self, output: &mut impl Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl Tagged for ValueToA {}

impl<I: PointInput<Extra = (Arc<TailSchema>, Arc<TailSchema>)>> Parse<I> for ValueToA
where
    TailValue: Parse<I::WithExtra<Arc<TailSchema>>>,
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let (mut input, n) = input.remaining()?;
        let (a, b) = input.extra().clone();
        let ae = a
            .item_size()
            .ok_or_else(|| object_rainbow::error_parse!("no item size"))?;
        let be = b
            .item_size()
            .ok_or_else(|| object_rainbow::error_parse!("no item size"))?;
        let k = ae
            .checked_add(be)
            .ok_or_else(|| object_rainbow::error_parse!("no item size"))?;
        let n = try_divide(
            n,
            k.try_into()
                .map_err(|_| object_rainbow::Error::UnsupportedLength)?,
        )?
        .checked_mul(
            ae.try_into()
                .map_err(|_| object_rainbow::Error::UnsupportedLength)?,
        )
        .ok_or(object_rainbow::Error::UnsupportedLength)?;
        Ok(Self(input.split_parse(n)?, input.parse()?))
    }
}

#[derive(Debug, ToOutput, ParseAsInline, ListHashes, Topological, PartialEq)]
#[rainbow(untagged)]
pub enum InlineValue {
    Unit,
    Option(ValueOption<Self>),
    #[cfg(feature = "point")]
    Point(ValuePoint),
    Nt(ValueNt),
    Concat(Arc<Self>, Arc<Self>),
    Array(ValueArray),
    Numeric(NumericValue),
    Enum(EnumValue<Self>),
    #[cfg(feature = "_collections")]
    Collection(CollectionValue),
    Zt(ValueZt),
}

impl InlineOutput for InlineValue {}
impl Tagged for InlineValue {}

#[derive(Debug, ToOutput, ListHashes, Topological, PartialEq)]
#[rainbow(untagged)]
pub enum TailValue {
    Cut,
    Option(ValueOption<Self>),
    Sequence(ValueSequence),
    Concat(Arc<InlineValue>, Arc<Self>),
    ToA(ValueToA),
    Enum(EnumValue<Self>),
    Bytes(Vec<u8>),
    String(String),
}

impl Tagged for TailValue {}

#[derive(Clone)]
pub enum SchemaNiche {
    Zeroes(usize),
    ZeroNoNiche(usize),
    DecrByte(u8),
    AndNiche(Arc<Self>, Arc<Self>),
    NicheAnd(Arc<Self>, Arc<Self>),
    NoNiche2(Arc<Self>, Arc<Self>),
    HashNiche(u128),
    Cut,
    Repeat(Arc<Self>, u64),
}

impl ToOutput for SchemaNiche {
    fn to_output(&self, output: &mut impl Output) {
        match self {
            Self::Zeroes(_) => {}
            Self::ZeroNoNiche(_) => {}
            Self::DecrByte(n) => n.to_output(output),
            Self::AndNiche(a, b) => (a, b).to_output(output),
            Self::NicheAnd(a, b) => (a, b).to_output(output),
            Self::NoNiche2(a, b) => (a, b).to_output(output),
            Self::HashNiche(n) => {
                0u128.to_output(output);
                n.to_output(output);
            }
            Self::Cut => {}
            Self::Repeat(niche, n) => {
                for _ in 0..*n {
                    niche.to_output(output);
                }
            }
        }
    }
}

impl InlineOutput for SchemaNiche {}

impl SchemaNiche {
    pub fn point() -> Self {
        Self::HashNiche(u128::MAX)
    }

    pub fn needs_tag(&self) -> bool {
        match self {
            Self::Zeroes(_) => false,
            Self::ZeroNoNiche(_) => true,
            Self::DecrByte(_) => false,
            Self::AndNiche(_, _) => false,
            Self::NicheAnd(_, _) => false,
            Self::NoNiche2(_, _) => true,
            Self::HashNiche(_) => false,
            Self::Cut => true,
            Self::Repeat(_, _) => true,
        }
    }

    pub fn cut(&self) -> bool {
        match self {
            Self::Zeroes(_) => false,
            Self::ZeroNoNiche(_) => false,
            Self::DecrByte(_) => false,
            Self::AndNiche(a, b) => a.cut() || b.cut(),
            Self::NicheAnd(a, b) => a.cut() || b.cut(),
            Self::NoNiche2(a, b) => a.cut() || b.cut(),
            Self::HashNiche(_) => false,
            Self::Cut => true,
            Self::Repeat(_, _) => false,
        }
    }

    pub fn concat(a: Arc<Self>, b: Arc<Self>) -> Self {
        if a.cut() {
            (*a).clone()
        } else if a.needs_tag() {
            if b.needs_tag() {
                Self::NoNiche2(a, b)
            } else {
                Self::AndNiche(a, b)
            }
        } else {
            Self::NicheAnd(a, b)
        }
    }

    pub fn stop(self) -> Self {
        if self.cut() {
            self
        } else {
            Self::concat(Arc::new(self), Arc::new(Self::Cut))
        }
    }

    pub fn repeat(self, n: u64) -> Self {
        if n == 0 {
            Self::ZeroNoNiche(0)
        } else if n == 1 || self.cut() {
            self
        } else if self.needs_tag() {
            Self::Repeat(Arc::new(self), n)
        } else {
            self.stop()
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Zeroes(n) => Self::ZeroNoNiche(*n),
            Self::ZeroNoNiche(n) => Self::ZeroNoNiche(*n),
            Self::DecrByte(0) => Self::ZeroNoNiche(1),
            Self::DecrByte(n) => Self::DecrByte(*n - 1),
            Self::AndNiche(a, b) => Self::concat(a.clone(), Arc::new(b.next())),
            Self::NicheAnd(a, b) => Self::concat(Arc::new(a.next()), b.clone()),
            Self::NoNiche2(a, b) => Self::NoNiche2(a.clone(), b.clone()),
            Self::HashNiche(0) => Self::ZeroNoNiche(32),
            Self::HashNiche(n) => Self::HashNiche(*n - 1),
            Self::Cut => Self::Cut,
            Self::Repeat(niche, n) => Self::Repeat(niche.clone(), *n),
        }
    }

    pub fn option(&self) -> Self {
        if self.needs_tag() {
            Self::DecrByte(0xfd).stop()
        } else {
            self.next()
        }
    }
}

impl AbstractSchema for InlineSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::Never => SchemaNiche::Zeroes(0),
            Self::Unit => SchemaNiche::ZeroNoNiche(0),
            Self::Option(schema) => schema.niche().option(),
            Self::Point(_) => SchemaNiche::point(),
            Self::Nt(schema) => Self::Option(schema.clone()).niche(),
            Self::Concat(a, b) => SchemaNiche::concat(Arc::new(a.niche()), Arc::new(b.niche())),
            Self::Array(schema) => schema.niche(),
            Self::Numeric(schema) => schema.niche(),
            Self::Enum(schema) => schema.niche(),
            Self::Collection(schema) => schema.niche(),
            Self::Zt(_) => SchemaNiche::Cut,
        }
    }
}

impl OptionSchema for InlineSchema {
    fn option(self: Arc<Self>) -> Self {
        Self::Option(self)
    }
}

impl DefaultSchema<InlineValue> for InlineSchema {
    fn default_value(&self) -> Option<InlineValue> {
        match self {
            Self::Never => None,
            Self::Unit => Some(InlineValue::Unit),
            Self::Option(schema) => Some(InlineValue::Option(ValueOption::None(schema.clone()))),
            #[cfg(feature = "point")]
            Self::Point(schema) => schema.default_value().map(From::from),
            #[cfg(not(feature = "point"))]
            Self::Point(_) => None,
            Self::Nt(schema) => Some(InlineValue::Nt(ValueNt {
                items: Default::default(),
                schema: schema.clone(),
            })),
            Self::Concat(a, b) => Some(InlineValue::Concat(
                Arc::new(a.default_value()?),
                Arc::new(b.default_value()?),
            )),
            Self::Array(schema) => schema.default_value().map(From::from),
            Self::Numeric(schema) => schema.default_value().map(From::from),
            Self::Enum(schema) => schema.default_value().map(From::from),
            #[cfg(feature = "_collections")]
            Self::Collection(schema) => schema.default_value().map(From::from),
            #[cfg(not(feature = "_collections"))]
            Self::Collection(_) => None,
            Self::Zt(schema) => ValueZt::schema_default(schema.clone()).map(From::from),
        }
    }
}

impl DefaultIsMin for InlineSchema {
    fn default_is_min(&self) -> bool {
        match self {
            Self::Never => false,
            Self::Unit => true,
            Self::Option(_) => true,
            Self::Point(_) => false,
            Self::Nt(_) => true,
            Self::Concat(a, b) => a.default_is_min() && b.default_is_min(),
            Self::Array(schema) => schema.default_is_min(),
            Self::Numeric(schema) => schema.default_is_min(),
            Self::Enum(schema) => schema.default_is_min(),
            Self::Collection(schema) => schema.default_is_min(),
            Self::Zt(schema) => schema.default_is_min(),
        }
    }
}

impl AbstractValue for InlineValue {
    type Schema = InlineSchema;
    fn schema(&self) -> Self::Schema {
        match self {
            Self::Unit => InlineSchema::Unit,
            Self::Option(o) => o.schema(),
            #[cfg(feature = "point")]
            Self::Point(p) => p.schema().into(),
            Self::Nt(nt) => nt.schema(),
            Self::Concat(a, b) => InlineSchema::Concat(Arc::new(a.schema()), Arc::new(b.schema())),
            Self::Array(a) => a.schema().into(),
            Self::Numeric(n) => n.schema().into(),
            Self::Enum(e) => e.schema().into(),
            #[cfg(feature = "_collections")]
            Self::Collection(c) => c.schema().into(),
            Self::Zt(z) => z.schema(),
        }
    }
}

impl<T: AbstractValue> ToOutput for ValueOption<T> {
    fn to_output(&self, output: &mut impl Output) {
        match self {
            Self::None(schema) => {
                let niche = schema.niche();
                if niche.needs_tag() {
                    0xfeu8.to_output(output);
                } else {
                    niche.to_output(output);
                }
            }
            Self::Some(value) => {
                if value.schema().niche().needs_tag() {
                    0xffu8.to_output(output);
                }
                value.to_output(output);
            }
        }
    }
}

impl<T: AbstractValue + InlineOutput> InlineOutput for ValueOption<T> {}

impl<T: AbstractValue> ValueOption<T> {
    pub fn inner_schema(&self) -> Arc<T::Schema> {
        match self {
            Self::None(schema) => schema.clone(),
            Self::Some(value) => Arc::new(value.schema()),
        }
    }
}

impl<T: AbstractValue + Parse<I>, I: PointInput<Extra = Arc<T::Schema>>> Parse<I>
    for ValueOption<T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        let niche = schema.niche();
        if niche.needs_tag() {
            match input.parse_inline::<u8>()? {
                0xff => Ok(Self::Some(input.parse()?)),
                0xfe => Ok(Self::None(schema)),
                _ => Err(object_rainbow::Error::OutOfBounds),
            }
        } else {
            input.parse_compare(&niche.vec()).map(|value| match value {
                Some(value) => Self::Some(value),
                None => Self::None(schema),
            })
        }
    }
}

impl<T: AbstractValue + ParseInline<I>, I: PointInput<Extra = Arc<T::Schema>>> ParseInline<I>
    for ValueOption<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        let niche = schema.niche();
        if niche.needs_tag() {
            match input.parse_inline::<u8>()? {
                0xff => Ok(Self::Some(input.parse_inline()?)),
                0xfe => Ok(Self::None(schema)),
                _ => Err(object_rainbow::Error::OutOfBounds),
            }
        } else {
            input
                .parse_compare_inline(&niche.vec())
                .map(|value| match value {
                    Some(value) => Self::Some(value),
                    None => Self::None(schema),
                })
        }
    }
}

impl<T: AbstractValue<Schema: OptionSchema>> AbstractValue for ValueOption<T> {
    type Schema = T::Schema;

    fn schema(&self) -> Self::Schema {
        self.inner_schema().option()
    }
}

impl AbstractSchema for TailSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::Cut => SchemaNiche::Cut,
            Self::Option(schema) => schema.niche().option(),
            Self::Sequence(_) => SchemaNiche::Cut,
            Self::Concat(a, b) => SchemaNiche::concat(Arc::new(a.niche()), Arc::new(b.niche())),
            Self::ToA(_, _) => SchemaNiche::Cut,
            Self::Enum(schema) => schema.niche(),
            Self::Bytes => SchemaNiche::Cut,
            Self::String => SchemaNiche::Cut,
        }
    }
}

impl OptionSchema for TailSchema {
    fn option(self: Arc<Self>) -> Self {
        Self::Option(self)
    }
}

impl DefaultSchema<TailValue> for TailSchema {
    fn default_value(&self) -> Option<TailValue> {
        match self {
            Self::Cut => Some(TailValue::Cut),
            Self::Option(schema) => Some(TailValue::Option(ValueOption::None(schema.clone()))),
            Self::Sequence(schema) => Some(TailValue::Sequence(ValueSequence {
                schema: Extras(schema.clone()),
                items: Default::default(),
            })),
            Self::Concat(a, b) => Some(TailValue::Concat(
                Arc::new(a.default_value()?),
                Arc::new(b.default_value()?),
            )),
            Self::ToA(a, b) => Some(TailValue::ToA(ValueToA(
                Arc::new(a.default_value()?).into(),
                Arc::new(b.default_value()?).into(),
            ))),
            Self::Enum(schema) => schema.default_value().map(From::from),
            Self::Bytes => Some(TailValue::Bytes(Default::default())),
            Self::String => Some(TailValue::String(Default::default())),
        }
    }
}

impl DefaultIsMin for TailSchema {
    fn default_is_min(&self) -> bool {
        match self {
            Self::Cut => true,
            Self::Option(_) => true,
            Self::Sequence(_) => true,
            Self::Concat(a, b) => a.default_is_min() && b.default_is_min(),
            Self::ToA(_, _) => false,
            Self::Enum(schema) => schema.default_is_min(),
            Self::Bytes => true,
            Self::String => true,
        }
    }
}

impl AbstractValue for TailValue {
    type Schema = TailSchema;
    fn schema(&self) -> Self::Schema {
        match self {
            Self::Cut => TailSchema::Cut,
            Self::Option(o) => o.schema(),
            Self::Sequence(s) => s.schema(),
            Self::Concat(a, b) => TailSchema::Concat(Arc::new(a.schema()), Arc::new(b.schema())),
            Self::ToA(ValueToA(a, b)) => {
                TailSchema::ToA(Arc::new(a.schema()), Arc::new(b.schema()))
            }
            Self::Enum(e) => e.schema().into(),
            Self::Bytes(_) => TailSchema::Bytes,
            Self::String(_) => TailSchema::String,
        }
    }
}

impl<
    I: PointInput<
            Extra = Arc<InlineSchema>,
            WithExtra<Arc<InlineSchema>> = I,
            WithExtra<Arc<TailSchema>> = J,
        >,
    J: PointInput<
            Extra = Arc<TailSchema>,
            WithExtra<Arc<InlineSchema>> = I,
            WithExtra<Arc<TailSchema>> = J,
        >,
> ParseInline<I> for InlineValue
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(match &*schema {
            InlineSchema::Never => match input.parse_inline::<Infallible>()? {},
            InlineSchema::Unit => Self::Unit,
            InlineSchema::Option(schema) => Self::Option(input.parse_inline_extra(schema.clone())?),
            #[cfg(feature = "point")]
            InlineSchema::Point(schema) => {
                Self::Point(input.parse_inline_extra(schema.clone().schema)?)
            }
            #[cfg(not(feature = "point"))]
            InlineSchema::Point(i) => match *i {},
            InlineSchema::Nt(schema) => Self::Nt(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Concat(a, b) => Self::Concat(
                input.parse_inline_extra(a.clone())?,
                input.parse_inline_extra(b.clone())?,
            ),
            InlineSchema::Array(schema) => Self::Array(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Numeric(schema) => {
                Self::Numeric(input.parse_inline_extra(schema.clone())?)
            }
            InlineSchema::Enum(schema) => Self::Enum(input.parse_inline_extra(schema.clone())?),
            #[cfg(feature = "_collections")]
            InlineSchema::Collection(schema) => {
                Self::Collection(input.parse_inline_extra(schema.clone())?)
            }
            #[cfg(not(feature = "_collections"))]
            InlineSchema::Collection(i) => match *i {},
            InlineSchema::Zt(schema) => Self::Zt(input.parse_inline_extra(schema.clone())?),
        })
    }
}

impl<I: PointInput<Extra = Arc<TailSchema>, WithExtra<Arc<TailSchema>> = I>> Parse<I> for TailValue
where
    InlineValue: ParseInline<I::WithExtra<Arc<InlineSchema>>>,
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(match &*schema {
            TailSchema::Cut => Self::Cut,
            TailSchema::Option(schema) => Self::Option(input.parse_extra(schema.clone())?),
            TailSchema::Sequence(schema) => Self::Sequence(input.parse_extra(schema.clone())?),
            TailSchema::Concat(a, b) => Self::Concat(
                input.parse_inline_extra(a.clone())?,
                input.parse_extra(b.clone())?,
            ),
            TailSchema::ToA(a, b) => Self::ToA(input.parse_extra((a.clone(), b.clone()))?),
            TailSchema::Enum(schema) => Self::Enum(input.parse_extra(schema.clone())?),
            TailSchema::Bytes => Self::Bytes(input.parse()?),
            TailSchema::String => Self::Bytes(input.parse()?),
        })
    }
}

#[derive(
    Debug, ToOutput, InlineOutput, Parse, ParseInline, ListHashes, Topological, Tagged, PartialEq,
)]
pub struct EnumSchema<T> {
    pub kind: NumericSchema,
    pub variants: Arc<LpVec<Arc<T>>>,
}

impl From<EnumSchema<InlineSchema>> for InlineSchema {
    fn from(schema: EnumSchema<InlineSchema>) -> Self {
        Self::Enum(schema)
    }
}

impl From<EnumSchema<TailSchema>> for TailSchema {
    fn from(schema: EnumSchema<TailSchema>) -> Self {
        Self::Enum(schema)
    }
}

impl<T> Clone for EnumSchema<T> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl<T: AbstractSchema> AbstractSchema for EnumSchema<T> {
    fn niche(&self) -> SchemaNiche {
        self.kind.niche().stop()
    }
}

impl<T: AbstractValue<Schema: DefaultSchema<T>>> DefaultSchema<EnumValue<T>>
    for EnumSchema<T::Schema>
{
    fn default_value(&self) -> Option<EnumValue<T>> {
        Some(EnumValue {
            kind: self.kind.default_value()?,
            variants: self.variants.clone(),
            value: Arc::new(self.variants.first()?.default_value()?),
        })
    }
}

impl<T: DefaultIsMin> DefaultIsMin for EnumSchema<T> {
    fn default_is_min(&self) -> bool {
        self.kind.default_is_min()
            && self
                .variants
                .first()
                .is_some_and(|schema| schema.default_is_min())
    }
}

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Tagged, PartialEq)]
pub struct EnumValue<T: AbstractValue> {
    pub kind: NumericValue,
    pub variants: Arc<LpVec<Arc<T::Schema>>>,
    pub value: Arc<T>,
}

impl From<EnumValue<InlineValue>> for InlineValue {
    fn from(value: EnumValue<InlineValue>) -> Self {
        Self::Enum(value)
    }
}

impl From<EnumValue<TailValue>> for TailValue {
    fn from(value: EnumValue<TailValue>) -> Self {
        Self::Enum(value)
    }
}

impl<T: AbstractValue> AbstractValue for EnumValue<T> {
    type Schema = EnumSchema<T::Schema>;

    fn schema(&self) -> Self::Schema {
        EnumSchema {
            kind: self.kind.schema(),
            variants: self.variants.clone(),
        }
    }
}

impl<
    T: AbstractValue + Parse<I::WithExtra<Arc<T::Schema>>>,
    I: PointInput<Extra = EnumSchema<T::Schema>>,
> Parse<I> for EnumValue<T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let EnumSchema { kind, variants } = input.extra().clone();
        let kind: NumericValue = input.parse_inline_extra(kind.clone())?;
        let schema = variants
            .get(kind.index().ok_or(object_rainbow::Error::OutOfBounds)?)
            .ok_or(object_rainbow::Error::OutOfBounds)?
            .clone();
        let value = input.parse_extra(schema)?;
        Ok(Self {
            kind,
            variants,
            value,
        })
    }
}

impl<
    T: AbstractValue + ParseInline<I::WithExtra<Arc<T::Schema>>>,
    I: PointInput<Extra = EnumSchema<T::Schema>>,
> ParseInline<I> for EnumValue<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let EnumSchema { kind, variants } = input.extra().clone();
        let kind: NumericValue = input.parse_inline_extra(kind.clone())?;
        let schema = variants
            .get(kind.index().ok_or(object_rainbow::Error::OutOfBounds)?)
            .ok_or(object_rainbow::Error::OutOfBounds)?
            .clone();
        let value = input.parse_inline_extra(schema)?;
        Ok(Self {
            kind,
            variants,
            value,
        })
    }
}

assert_impl!(
    impl Inline for InlineSchema {}
);
assert_impl!(
    impl ReflessInline for InlineSchema {}
);
assert_impl!(
    impl Inline for TailSchema {}
);
assert_impl!(
    impl ReflessInline for TailSchema {}
);

impl SizeSchema for ArraySchema {
    fn size(&self) -> Option<u64> {
        self.len.checked_mul(self.schema.size()?)
    }
}

impl<T: SizeSchema> SizeSchema for EnumSchema<T> {
    fn size(&self) -> Option<u64> {
        let size = self.variants.first()?.size()?;
        for schema in &self.variants[1..] {
            if schema.size()? != size {
                return None;
            }
        }
        self.kind.size()?.checked_add(size)
    }
}

impl SizeSchema for InlineSchema {
    fn size(&self) -> Option<u64> {
        match self {
            Self::Never => Some(0),
            Self::Unit => Some(0),
            Self::Option(schema) => {
                if schema.niche().needs_tag() {
                    if schema.size()? == 0 { Some(1) } else { None }
                } else {
                    schema.size()
                }
            }
            Self::Point(_) => Some(32),
            Self::Nt(_) => None,
            Self::Concat(a, b) => a.size()?.checked_add(b.size()?),
            Self::Array(schema) => schema.size(),
            Self::Numeric(schema) => schema.size(),
            Self::Enum(schema) => schema.size(),
            #[cfg(feature = "_collections")]
            Self::Collection(schema) => schema.size(),
            #[cfg(not(feature = "_collections"))]
            Self::Collection(i) => match *i {},
            Self::Zt(_) => None,
        }
    }
}

impl ItemSizeSchema for TailSchema {
    fn item_size(&self) -> Option<u64> {
        match self {
            Self::Cut => None,
            Self::Option(_) => None,
            Self::Sequence(schema) => schema.size(),
            Self::Concat(_, _) => None,
            Self::ToA(a, b) => a.item_size()?.checked_add(b.item_size()?),
            Self::Enum(_) => None,
            Self::Bytes => Some(1),
            Self::String => None,
        }
    }
}

#[derive(Debug, ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline, PartialEq)]
pub struct ValueZt {
    pub schema: Extras<Arc<TailSchema>>,
    pub value: Zt<Arc<TailValue>>,
}

impl Tagged for ValueZt {}

impl ValueZt {
    pub fn schema_default(schema: Arc<TailSchema>) -> Option<Self> {
        let value = Zt::new(Arc::new(schema.default_value()?)).ok()?;
        Some(Self {
            schema: Extras(schema),
            value,
        })
    }
}

impl AbstractValue for ValueZt {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Zt(self.schema.0.clone())
    }
}

impl From<ValueZt> for InlineValue {
    fn from(value: ValueZt) -> Self {
        Self::Zt(value)
    }
}

impl<T: AbstractValue + AbstractCollection> AbstractCollection for ValueOption<T> {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        match self {
            Self::None(_) => Vec::new(),
            Self::Some(value) => value.items(),
        }
    }
}

impl<T: AbstractValue + AbstractCollection> AbstractCollection for EnumValue<T> {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.value.items()
    }
}

impl AbstractCollection for ValueZt {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        self.value.items()
    }
}

impl AbstractCollection for InlineValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        match self {
            Self::Unit => Vec::new(),
            Self::Option(value) => value.items(),
            #[cfg(feature = "point")]
            Self::Point(_) => Vec::new(),
            Self::Nt(value) => value.items(),
            Self::Concat(a, b) => [a.items(), b.items()].concat(),
            Self::Array(value) => value.items(),
            Self::Numeric(_) => Vec::new(),
            Self::Enum(value) => value.items(),
            #[cfg(feature = "_collections")]
            Self::Collection(_) => Vec::new(),
            Self::Zt(value) => value.items(),
        }
    }
}

impl AbstractCollection for TailValue {
    fn items(&self) -> Vec<Arc<InlineValue>> {
        match self {
            Self::Cut => Vec::new(),
            Self::Option(value) => value.items(),
            Self::Sequence(value) => value.items(),
            Self::Concat(a, b) => [a.items(), b.items()].concat(),
            Self::ToA(ValueToA(a, b)) => a
                .items()
                .into_iter()
                .zip(b.items())
                .map(|(a, b)| InlineValue::Concat(a, b))
                .map(Arc::new)
                .collect(),
            Self::Enum(value) => value.items(),
            Self::Bytes(bytes) => bytes
                .iter()
                .copied()
                .map(NumericValue::U8)
                .map(InlineValue::Numeric)
                .map(Arc::new)
                .collect(),
            Self::String(string) => string
                .chars()
                .map(NumericValue::OpaqueChar)
                .map(InlineValue::Numeric)
                .map(Arc::new)
                .collect(),
        }
    }
}

#[test]
fn tuple_of_arrays() -> object_rainbow::Result<()> {
    use object_rainbow::{ParseSlice, ParseSliceExtra};
    let resolve = &(Arc::new(object_rainbow::TopoVec::new()) as _);
    let schema = TailSchema::parse_slice(&[4, 2, 7, 0, 2, 7, 2], resolve)?;
    let value = TailValue::parse_slice_extra(&[1, 2, 3, 4, 5, 6], resolve, &Arc::new(schema))?;
    assert_eq!(
        value,
        TailValue::ToA(ValueToA(
            MappedExtra(
                Default::default(),
                Arc::new(TailValue::Sequence(ValueSequence {
                    schema: Extras(Arc::new(InlineSchema::Numeric(NumericSchema::U8))),
                    items: vec![
                        Arc::new(InlineValue::Numeric(NumericValue::U8(1))),
                        Arc::new(InlineValue::Numeric(NumericValue::U8(2))),
                    ]
                })),
            ),
            MappedExtra(
                Default::default(),
                Arc::new(TailValue::Sequence(ValueSequence {
                    schema: Extras(Arc::new(InlineSchema::Numeric(NumericSchema::U16))),
                    items: vec![
                        Arc::new(InlineValue::Numeric(NumericValue::U16(0x0304))),
                        Arc::new(InlineValue::Numeric(NumericValue::U16(0x0506))),
                    ]
                })),
            ),
        )),
    );
    assert_eq!(
        value.items(),
        vec![
            Arc::new(InlineValue::Concat(
                Arc::new(InlineValue::Numeric(NumericValue::U8(1))),
                Arc::new(InlineValue::Numeric(NumericValue::U16(0x0304))),
            )),
            Arc::new(InlineValue::Concat(
                Arc::new(InlineValue::Numeric(NumericValue::U8(2))),
                Arc::new(InlineValue::Numeric(NumericValue::U16(0x0506))),
            )),
        ],
    );
    Ok(())
}

#[test]
fn toa_se_equiv() -> object_rainbow::Result<()> {
    use object_rainbow::{ParseSlice, ParseSliceExtra};
    let resolve = &(Arc::new(object_rainbow::TopoVec::new()) as _);
    let toa_schema = TailSchema::parse_slice(&[4, 2, 7, 0, 2, 7, 2], resolve)?;
    let toa_value =
        TailValue::parse_slice_extra(&[1, 2, 3, 4, 5, 6], resolve, &Arc::new(toa_schema))?;
    let seq_schema = TailSchema::parse_slice(&[2, 5, 7, 0, 7, 2], resolve)?;
    let seq_value =
        TailValue::parse_slice_extra(&[1, 3, 4, 2, 5, 6], resolve, &Arc::new(seq_schema))?;
    assert_eq!(toa_value.items(), seq_value.items());
    Ok(())
}
