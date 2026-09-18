use object_rainbow::pod;

use crate::Apply;

pub trait Collision<Diff: Send>: Send + Sized {
    type Output: Send;
    fn always_okay(diff: &Diff) -> bool;
    fn okay(self) -> Self::Output;
    fn check(self) -> object_rainbow::Result<Self::Output>;
}

#[pod]
pub struct NoCollisions<T>(pub T);

impl<D: Send, T: Apply<D, Output = X>, X: Collision<D, Output = O>, O: Send> Apply<D>
    for NoCollisions<T>
{
    type Output = O;

    async fn apply(&mut self, diff: D) -> object_rainbow::Result<Self::Output> {
        let always_okay = X::always_okay(&diff);
        let output = self.0.apply(diff).await?;
        if always_okay {
            Ok(output.okay())
        } else {
            output.check()
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not unique")]
pub struct NotUnique;

impl From<NotUnique> for object_rainbow::Error {
    fn from(error: NotUnique) -> Self {
        Self::operation(error)
    }
}

impl<K: Send, V: Send> Collision<(V, K)> for Option<V> {
    type Output = ();

    fn always_okay(_: &(V, K)) -> bool {
        false
    }

    fn okay(self) -> Self::Output {
        assert!(self.is_none());
    }

    fn check(self) -> object_rainbow::Result<Self::Output> {
        if self.is_none() {
            Ok(())
        } else {
            Err(NotUnique.into())
        }
    }
}

impl<K: Send, V: Send> Collision<(Option<V>, K)> for Option<V> {
    type Output = Option<V>;

    fn always_okay((value, _): &(Option<V>, K)) -> bool {
        value.is_none()
    }

    fn okay(self) -> Self::Output {
        self
    }

    fn check(self) -> object_rainbow::Result<Self::Output> {
        if self.is_none() {
            Ok(None)
        } else {
            Err(NotUnique.into())
        }
    }
}
