#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::sync::Arc;

pub enum Schema {
    Point(
        #[cfg(feature = "point")] Arc<Self>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
}
