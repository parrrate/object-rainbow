use std::{convert::Infallible, num::NonZero, sync::Arc};

use object_rainbow::{
    Enum, Inline, InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseAsInline,
    ParseInline, PointInput, ReflessInline, Tagged, ToOutput, Topological, Traversible,
    assert_impl, length_prefixed::LpVec,
};

#[cfg(feature = "point")]
use self::point::{PointSchema, ValuePoint};

#[cfg(feature = "collections")]
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

pub trait DefaultIsMin {
    fn default_is_min(&self) -> bool;
}

pub trait DefaultSchema<T: AbstractValue<Schema = Self>>: AbstractSchema {
    fn default_value(&self) -> Option<T>;
}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological, Clone)]
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
        })
    }
}

impl DefaultIsMin for NumericSchema {
    fn default_is_min(&self) -> bool {
        true
    }
}

impl From<NumericSchema> for InlineSchema {
    fn from(schema: NumericSchema) -> Self {
        Self::Numeric(schema)
    }
}

#[derive(ToOutput, ListHashes, Topological, ParseAsInline)]
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
        })
    }
}

impl From<NumericValue> for InlineValue {
    fn from(value: NumericValue) -> Self {
        Self::Numeric(value)
    }
}

#[derive(
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    MaybeHasNiche,
    ListHashes,
    Topological,
    Tagged,
    Clone,
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

impl From<ArraySchema> for InlineSchema {
    fn from(schema: ArraySchema) -> Self {
        Self::Array(schema)
    }
}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological, Clone)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum InlineSchema {
    Never,
    Unit,
    Option(Arc<Self>),
    Point(
        #[cfg(feature = "point")] Arc<TailSchema>,
        #[cfg(not(feature = "point"))] std::convert::Infallible,
    ),
    Nt(Arc<Self>),
    Concat(Arc<Self>, Arc<Self>),
    Array(ArraySchema),
    Numeric(NumericSchema),
    Enum(EnumSchema<Self>),
}

impl InlineOutput for InlineSchema {}
impl Tagged for InlineSchema {}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological, Clone)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum TailSchema {
    Cut,
    Option(Arc<Self>),
    Sequence(Arc<InlineSchema>),
    Concat(Arc<InlineSchema>, Arc<Self>),
    Enum(EnumSchema<Self>),
}

impl InlineOutput for TailSchema {}
impl Tagged for TailSchema {}

#[derive(ListHashes, Topological, Tagged)]
#[rainbow(untagged)]
pub enum ValueOption<T: AbstractValue> {
    None(Arc<T::Schema>),
    Some(Arc<T>),
}

#[derive(ParseAsInline, ListHashes, Topological)]
pub struct ValueNt {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
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

#[derive(ParseAsInline, ListHashes, Topological)]
pub struct ValueArray {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
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

#[derive(ListHashes, Topological)]
pub struct ValueSequence {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl ToOutput for ValueSequence {
    fn to_output(&self, output: &mut impl Output) {
        self.items.to_output(output);
    }
}

impl Tagged for ValueSequence {}

impl AbstractValue for ValueSequence {
    type Schema = TailSchema;

    fn schema(&self) -> Self::Schema {
        TailSchema::Sequence(self.schema.clone())
    }
}

impl<I: PointInput<Extra = Arc<InlineSchema>>> Parse<I> for ValueSequence
where
    InlineValue: ParseInline<I>,
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(Self {
            items: input.parse()?,
            schema,
        })
    }
}

#[derive(ToOutput, ParseAsInline, ListHashes, Topological)]
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
}

impl InlineOutput for InlineValue {}
impl Tagged for InlineValue {}

#[derive(ToOutput, ListHashes, Topological)]
#[rainbow(untagged)]
pub enum TailValue {
    Cut,
    Option(ValueOption<Self>),
    Sequence(ValueSequence),
    Concat(Arc<InlineValue>, Arc<Self>),
    Enum(EnumValue<Self>),
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
    PointNiche(u128),
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
            Self::PointNiche(n) => {
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
        Self::PointNiche(u128::MAX)
    }

    pub fn needs_tag(&self) -> bool {
        match self {
            Self::Zeroes(_) => false,
            Self::ZeroNoNiche(_) => true,
            Self::DecrByte(_) => false,
            Self::AndNiche(_, _) => false,
            Self::NicheAnd(_, _) => false,
            Self::NoNiche2(_, _) => true,
            Self::PointNiche(_) => false,
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
            Self::PointNiche(_) => false,
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
            Self::PointNiche(0) => Self::ZeroNoNiche(32),
            Self::PointNiche(n) => Self::PointNiche(*n - 1),
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
            Self::Point(schema) => PointSchema {
                schema: schema.clone(),
            }
            .default_value()
            .map(From::from),
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
            Self::Enum(schema) => schema.niche(),
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
                items: Default::default(),
                schema: schema.clone(),
            })),
            Self::Concat(a, b) => Some(TailValue::Concat(
                Arc::new(a.default_value()?),
                Arc::new(b.default_value()?),
            )),
            Self::Enum(schema) => schema.default_value().map(From::from),
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
            Self::Enum(e) => e.schema().into(),
        }
    }
}

impl<I: PointInput<Extra = Arc<InlineSchema>, WithExtra<Arc<InlineSchema>> = I>> ParseInline<I>
    for InlineValue
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(match &*schema {
            InlineSchema::Never => match input.parse_inline::<Infallible>()? {},
            InlineSchema::Unit => Self::Unit,
            InlineSchema::Option(schema) => Self::Option(input.parse_inline_extra(schema.clone())?),
            #[cfg(feature = "point")]
            InlineSchema::Point(schema) => Self::Point(input.parse_inline_extra(schema.clone())?),
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
            TailSchema::Enum(schema) => Self::Enum(input.parse_extra(schema.clone())?),
        })
    }
}

#[derive(ToOutput, InlineOutput, Parse, ParseInline, ListHashes, Topological, Tagged)]
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

#[derive(ToOutput, InlineOutput, ListHashes, Topological, Tagged)]
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
