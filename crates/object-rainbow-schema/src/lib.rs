use std::convert::Infallible;
use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput};

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[niche(tag)]
#[parse(unchecked)]
pub enum Schema {
    Never(Infallible),
    Option(Arc<Self>),
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
}

impl InlineOutput for Schema {}
impl Tagged for Schema {}
