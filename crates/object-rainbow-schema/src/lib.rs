#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::sync::Arc;

use object_rainbow::Enum;

#[derive(Enum)]
#[enumtag("char")]
pub enum Schema {
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
}
