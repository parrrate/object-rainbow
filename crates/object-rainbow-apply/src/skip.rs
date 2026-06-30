use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological,
};

use crate::Apply;

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
pub struct FilterDiffs<T>(pub T);

impl<T: Apply<D>, D: Send> Apply<(bool, D)> for FilterDiffs<T> {
    type Output = Option<T::Output>;

    async fn apply(&mut self, (include, diff): (bool, D)) -> object_rainbow::Result<Self::Output> {
        Ok(if include {
            Some(self.0.apply(diff).await?)
        } else {
            None
        })
    }
}
