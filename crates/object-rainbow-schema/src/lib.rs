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

#[derive(ToOutput)]
#[rainbow(untagged)]
pub enum Value {
    Unit,
    Option(ValueOption),
    #[cfg(feature = "point")]
    Point(Point<Self>),
}

impl InlineOutput for Value {}

pub enum DynNiche {
    Hash(u128),
}

impl Schema {
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
        }
    }

    pub fn needs_tag(&self, n: usize) -> bool {
        match self {
            Self::Never if n == 0 => unreachable!(),
            Self::Never => Self::Unit.needs_tag(n - 1),
            Self::Unit => n.is_multiple_of(255),
            Self::Option(schema) => schema.needs_tag(n + 1),
            Self::Point(_) => false,
            Self::Nt(schema) => Self::Option(schema.clone()).needs_tag(n),
        }
    }

    pub fn some_prefix(&self, output: &mut impl Output) {
        if self.needs_tag(0) {
            [0xff].to_output(output);
        }
    }
}

impl Value {
    pub fn niche_schema(&self) -> Schema {
        match self {
            Self::Unit => Schema::Unit,
            Self::Option(o) => Schema::Option(match o {
                ValueOption::None(schema) => schema.clone(),
                ValueOption::Some(value) => Arc::new(value.niche_schema()),
            }),
            #[cfg(feature = "point")]
            Self::Point(_) => Schema::Point(Arc::new(Schema::Unit)),
        }
    }
}

impl ToOutput for ValueOption {
    fn to_output(&self, output: &mut impl Output) {
        match self {
            Self::None(schema) => schema.none(0, output),
            Self::Some(value) => {
                value.niche_schema().some_prefix(output);
                value.to_output(output);
            }
        }
    }
}
