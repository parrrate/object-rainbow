use std::{convert::Infallible, sync::Arc};

use object_rainbow::{
    Enum, InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseAsInline, ParseInline,
    PointInput, ReflessInline, Tagged, ToOutput, Topological, Traversible,
};
#[cfg(feature = "point")]
use object_rainbow_point::Point;

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

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological, Clone)]
#[enumtag("char")]
#[niche(tag)]
pub enum NumericSchema {
    U8,
}

impl InlineOutput for NumericSchema {}
impl Tagged for NumericSchema {}

impl AbstractSchema for NumericSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::U8 => SchemaNiche::ZeroNoNiche(1),
        }
    }
}

#[derive(ToOutput, ListHashes, Topological, ParseAsInline)]
#[rainbow(untagged)]
pub enum NumericValue {
    U8(u8),
}

impl AbstractValue for NumericValue {
    type Schema = NumericSchema;

    fn schema(&self) -> Self::Schema {
        match self {
            Self::U8(_) => NumericSchema::U8,
        }
    }
}

impl InlineOutput for NumericValue {}
impl Tagged for NumericValue {}

impl<I: PointInput<Extra = NumericSchema>> ParseInline<I> for NumericValue {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(match input.extra().clone() {
            NumericSchema::U8 => Self::U8(input.parse_inline()?),
        })
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

impl From<ArraySchema> for InlineSchema {
    fn from(schema: ArraySchema) -> Self {
        Self::Array(schema)
    }
}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological)]
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
}

impl InlineOutput for InlineSchema {}
impl Tagged for InlineSchema {}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche, ListHashes, Topological)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum TailSchema {
    Cut,
    Option(Arc<Self>),
    Sequence(Arc<InlineSchema>),
    Concat(Arc<InlineSchema>, Arc<Self>),
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

#[derive(ListHashes, Topological, ParseAsInline)]
pub struct ValuePoint {
    pub point: Point<TailValue>,
    pub schema: Arc<TailSchema>,
}

impl ToOutput for ValuePoint {
    fn to_output(&self, output: &mut impl Output) {
        self.point.to_output(output);
    }
}

impl InlineOutput for ValuePoint {}
impl Tagged for ValuePoint {}

impl AbstractValue for ValuePoint {
    type Schema = InlineSchema;

    fn schema(&self) -> Self::Schema {
        InlineSchema::Point(self.schema.clone())
    }
}

impl<I: PointInput<Extra = Arc<TailSchema>>> ParseInline<I> for ValuePoint {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(Self {
            point: input.parse_inline()?,
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
            Self::AndNiche(a, b) => !a.cut() && !b.cut(),
            Self::NicheAnd(a, b) => !a.cut() && !b.cut(),
            Self::NoNiche2(a, b) => !a.cut() && !b.cut(),
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

    pub fn repeat(self, n: u64) -> Self {
        if n == 0 {
            Self::ZeroNoNiche(0)
        } else if n == 1 || self.cut() {
            self
        } else if self.needs_tag() {
            Self::Repeat(Arc::new(self), n)
        } else {
            Self::concat(Arc::new(self), Arc::new(Self::Cut))
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
            Self::NicheAnd(Arc::new(Self::DecrByte(0xfd)), Arc::new(Self::Cut))
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
            Self::Point(_) => SchemaNiche::PointNiche(u128::MAX),
            Self::Nt(schema) => Self::Option(schema.clone()).niche(),
            Self::Concat(a, b) => SchemaNiche::concat(Arc::new(a.niche()), Arc::new(b.niche())),
            Self::Array(schema) => schema.niche(),
            Self::Numeric(schema) => schema.niche(),
        }
    }
}

impl OptionSchema for InlineSchema {
    fn option(self: Arc<Self>) -> Self {
        Self::Option(self)
    }
}

impl AbstractValue for InlineValue {
    type Schema = InlineSchema;
    fn schema(&self) -> Self::Schema {
        match self {
            Self::Unit => InlineSchema::Unit,
            Self::Option(o) => o.schema(),
            #[cfg(feature = "point")]
            Self::Point(p) => p.schema(),
            Self::Nt(nt) => nt.schema(),
            Self::Concat(a, b) => InlineSchema::Concat(Arc::new(a.schema()), Arc::new(b.schema())),
            Self::Array(a) => a.schema().into(),
            Self::Numeric(n) => InlineSchema::Numeric(n.schema()),
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
        }
    }
}

impl OptionSchema for TailSchema {
    fn option(self: Arc<Self>) -> Self {
        Self::Option(self)
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
        }
    }
}

impl<I: PointInput<Extra = Arc<InlineSchema>, WithExtra<Arc<InlineSchema>> = I>> ParseInline<I>
    for InlineValue
where
    ValuePoint: ParseInline<I::WithExtra<Arc<TailSchema>>>,
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let schema = input.extra().clone();
        Ok(match &*schema {
            InlineSchema::Never => match input.parse_inline::<Infallible>()? {},
            InlineSchema::Unit => Self::Unit,
            InlineSchema::Option(schema) => Self::Option(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Point(schema) => Self::Point(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Nt(schema) => Self::Nt(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Concat(a, b) => Self::Concat(
                input.parse_inline_extra(a.clone())?,
                input.parse_inline_extra(b.clone())?,
            ),
            InlineSchema::Array(schema) => Self::Array(input.parse_inline_extra(schema.clone())?),
            InlineSchema::Numeric(schema) => {
                Self::Numeric(input.parse_inline_extra(schema.clone())?)
            }
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
        })
    }
}
