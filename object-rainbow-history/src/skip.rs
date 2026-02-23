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

    fn apply(
        &mut self,
        (include, diff): (bool, D),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move {
            Ok(if include {
                Some(self.0.apply(diff).await?)
            } else {
                None
            })
        }
    }
}
