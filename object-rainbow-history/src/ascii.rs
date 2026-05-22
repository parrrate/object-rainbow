use object_rainbow::{InlineOutput, ListHashes, Tagged, ToOutput, Topological};

use crate::Apply;

#[derive(Debug, Clone, Copy, ToOutput, InlineOutput, Tagged, ListHashes, Topological)]
pub struct Split;

impl<S: 'static + Send + AsRef<str>> Apply<S> for Split {
    type Output = Vec<String>;

    fn apply(
        &mut self,
        diff: S,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        core::future::ready(Ok(diff
            .as_ref()
            .split_ascii_whitespace()
            .map(String::from)
            .collect()))
    }
}
