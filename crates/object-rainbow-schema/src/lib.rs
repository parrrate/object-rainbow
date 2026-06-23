#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::sync::Arc;

use object_rainbow::{Enum, Parse, ToOutput};

#[derive(Enum, ToOutput, Parse)]
#[enumtag("char")]
#[parse(unchecked)]
pub enum Schema {
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
}
