use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, MaybeHasNiche, Output, Parse, ParseInline, Tagged, ToOutput,
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
}

impl InlineOutput for Schema {}
impl Tagged for Schema {}

pub enum ValueOption {
    None(Arc<Schema>),
    Some(Arc<Value>),
}

pub enum Value {
    Unit,
    Option(ValueOption),
    #[cfg(feature = "point")]
    Point(Point<Self>),
}

pub enum DynNiche {
    Hash(u128),
}

impl Schema {
    pub fn none(&self, n: usize, output: &mut impl Output) {
        match self {
            Self::Never if n == 0 => {}
            Self::Never => {
                Self::Unit.none(n - 1, output);
            }
            Self::Unit => {
                [255 - (n % 256) as u8].to_output(output);
            }
            Self::Option(schema) => schema.none(n + 1, output),
            Self::Point(_) => {
                0u128.to_output(output);
                (u128::MAX - (n as u128)).to_output(output);
            }
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
