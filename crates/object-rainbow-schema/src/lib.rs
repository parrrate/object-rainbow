use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, MaybeHasNiche, Output, Parse, ParseInline, Tagged, ToOutput,
    none_terminated::Nt,
};
#[cfg(feature = "point")]
use object_rainbow_point::Point;

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum Schema {
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

impl InlineOutput for Schema {}
impl Tagged for Schema {}

pub enum ValueOption {
    None(Arc<Schema>),
    Some(Arc<Value>),
}

pub struct ValueNt {
    pub items: Nt<Vec<Arc<Value>>>,
    pub schema: Arc<Schema>,
}

impl ToOutput for ValueNt {
    fn to_output(&self, output: &mut impl Output) {
        self.items.to_output(output);
    }
}

impl InlineOutput for ValueNt {}

pub struct ValuePoint {
    pub point: Point<Value>,
    pub schema: Arc<Schema>,
}

impl ToOutput for ValuePoint {
    fn to_output(&self, output: &mut impl Output) {
        self.point.to_output(output);
    }
}

impl InlineOutput for ValuePoint {}

#[derive(ToOutput)]
#[rainbow(untagged)]
pub enum Value {
    Unit,
    Option(ValueOption),
    #[cfg(feature = "point")]
    Point(ValuePoint),
    Nt(Arc<ValueNt>),
    Concat(Arc<Self>, Arc<Self>),
}

impl InlineOutput for Value {}

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
            SchemaNiche::Cut => false,
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
}

impl Schema {
    pub fn niche(&self) -> SchemaNiche {
        match self {
            Self::Never => SchemaNiche::Zeroes(0),
            Self::Unit => SchemaNiche::ZeroNoNiche(0),
            Self::Option(schema) => {
                let sub = schema.niche();
                if sub.needs_tag() {
                    SchemaNiche::NicheAnd(
                        Arc::new(SchemaNiche::DecrByte(0xfd)),
                        Arc::new(SchemaNiche::Cut),
                    )
                } else {
                    sub.next()
                }
            }
            Self::Point(_) => SchemaNiche::PointNiche(u128::MAX),
            Self::Nt(schema) => Self::Option(schema.clone()).niche(),
            Self::Concat(a, b) => SchemaNiche::concat(Arc::new(a.niche()), Arc::new(b.niche())),
        }
    }

    pub fn none(&self, n: usize, output: &mut impl Output) {
        match self {
            Self::Never if n == 0 => {}
            Self::Never => Self::Unit.none(n - 1, output),
            Self::Unit => {
                [254 - (n % 255) as u8].to_output(output);
            }
            Self::Option(schema) => schema.none(n + 1, output),
            Self::Point(_) => {
                0u128.to_output(output);
                (u128::MAX - (n as u128)).to_output(output);
            }
            Self::Nt(schema) => Self::Option(schema.clone()).none(n, output),
            Self::Concat(schema, _) => schema.none(n, output),
        }
    }
}

impl Value {
    pub fn schema(&self) -> Schema {
        match self {
            Self::Unit => Schema::Unit,
            Self::Option(o) => Schema::Option(match o {
                ValueOption::None(schema) => schema.clone(),
                ValueOption::Some(value) => Arc::new(value.schema()),
            }),
            #[cfg(feature = "point")]
            Self::Point(ValuePoint { schema, .. }) => Schema::Point(schema.clone()),
            Self::Nt(nt) => Schema::Nt(nt.schema.clone()),
            Self::Concat(a, b) => Schema::Concat(Arc::new(a.schema()), Arc::new(b.schema())),
        }
    }
}

impl ToOutput for ValueOption {
    fn to_output(&self, output: &mut impl Output) {
        match self {
            Self::None(schema) => schema.none(0, output),
            Self::Some(value) => {
                if value.schema().niche().needs_tag() {
                    0xffu8.to_output(output);
                }
                value.to_output(output);
            }
        }
    }
}
