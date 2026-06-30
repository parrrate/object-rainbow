use object_rainbow::{
    Fetch, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, Traversible,
};
use object_rainbow_point::Point;

use crate::Apply;

impl<T: Clone + Traversible + Apply<D>, D: Send> Apply<D> for Point<T> {
    type Output = T::Output;

    async fn apply(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        self.fetch_mut().await?.apply(diff).await
    }
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct Points<T>(pub T);

impl<T: Apply<D>, D: Send + Traversible> Apply<Point<D>> for Points<T> {
    type Output = T::Output;

    async fn apply(&mut self, diff: Point<D>) -> object_rainbow::Result<Self::Output> {
        self.0.apply(diff.fetch().await?).await
    }
}
