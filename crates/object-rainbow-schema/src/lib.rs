#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, MaybeHasNiche, Parse, ParseInline, Tagged, ToOutput};

#[derive(Enum, ToOutput, Parse, ParseInline, MaybeHasNiche)]
#[enumtag("char")]
#[parse(unchecked)]
#[niche(tag)]
pub enum Schema {
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
}

impl InlineOutput for Schema {}
impl Tagged for Schema {}
