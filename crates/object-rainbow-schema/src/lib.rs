use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput};

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum Schema {
    Never,
    Option(Arc<Self>),
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] std::convert::Infallible,
    ),
}

impl InlineOutput for Schema {}
impl Tagged for Schema {}

pub enum Value {
    Option(Option<Arc<Self>>),
}
