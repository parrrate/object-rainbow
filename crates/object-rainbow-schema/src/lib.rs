use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, MaybeHasNiche, Output, Parse, ParseInline, Tagged, ToOutput,
};
#[cfg(feature = "point")]
use object_rainbow_point::Point;

pub trait AbstractSchema {
    fn niche(&self) -> SchemaNiche;
}

pub trait OptionSchema: AbstractSchema {
    fn option(self: Arc<Self>) -> Self;
}

pub trait AbstractValue: ToOutput {
    type Schema: AbstractSchema;
    fn schema(&self) -> Self::Schema;
}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum InlineSchema {
    Never,
    Unit,
    Option(Arc<Self>),
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] std::convert::Infallible,
    ),
    Nt(Arc<Self>),
    Concat(Arc<Self>, Arc<Self>),
}

impl InlineOutput for InlineSchema {}
impl Tagged for InlineSchema {}

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum TailSchema {
    Cut,
    Sequence(Arc<InlineSchema>),
    Concat(Arc<InlineSchema>, Arc<Self>),
}

impl InlineOutput for TailSchema {}
impl Tagged for TailSchema {}

pub enum ValueOption<T: AbstractValue> {
    None(Arc<T::Schema>),
    Some(Arc<T>),
}

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

pub struct ValueSequence {
    pub items: Vec<Arc<InlineValue>>,
    pub schema: Arc<InlineSchema>,
}

impl ToOutput for ValueSequence {
    fn to_output(&self, output: &mut impl Output) {
        self.items.to_output(output);
    }
}

pub struct ValuePoint {
    pub point: Point<InlineValue>,
    pub schema: Arc<InlineSchema>,
}

impl ToOutput for ValuePoint {
    fn to_output(&self, output: &mut impl Output) {
        self.point.to_output(output);
    }
}

impl InlineOutput for ValuePoint {}

#[derive(ToOutput)]
#[rainbow(untagged)]
pub enum InlineValue {
    Unit,
    Option(ValueOption<Self>),
    #[cfg(feature = "point")]
    Point(ValuePoint),
    Nt(Arc<ValueNt>),
    Concat(Arc<Self>, Arc<Self>),
}

impl InlineOutput for InlineValue {}

#[derive(ToOutput)]
#[rainbow(untagged)]
pub enum TailValue {
    Cut,
    Sequence(ValueSequence),
    Concat(Arc<InlineValue>, Arc<Self>),
}

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
            Self::Point(ValuePoint { schema, .. }) => InlineSchema::Point(schema.clone()),
            Self::Nt(nt) => InlineSchema::Nt(nt.schema.clone()),
            Self::Concat(a, b) => InlineSchema::Concat(Arc::new(a.schema()), Arc::new(b.schema())),
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

impl<T: AbstractValue<Schema: OptionSchema>> AbstractValue for ValueOption<T> {
    type Schema = T::Schema;

    fn schema(&self) -> Self::Schema {
        match self {
            Self::None(schema) => schema.clone(),
            Self::Some(value) => Arc::new(value.schema()),
        }
        .option()
    }
}

impl AbstractSchema for TailSchema {
    fn niche(&self) -> SchemaNiche {
        match self {
            Self::Cut => SchemaNiche::Cut,
            Self::Sequence(_) => SchemaNiche::Cut,
            Self::Concat(a, b) => SchemaNiche::concat(Arc::new(a.niche()), Arc::new(b.niche())),
        }
    }
}
