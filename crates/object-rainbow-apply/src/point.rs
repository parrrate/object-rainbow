use object_rainbow::Traversible;
use object_rainbow_point::Point;

use crate::Apply;

impl<T: Clone + Traversible + Apply<D>, D: Send> Apply<D> for Point<T> {
    type Output = T::Output;

    async fn apply(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        self.fetch_mut().await?.apply(diff).await
    }
}
