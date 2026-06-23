use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput};
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
