use object_rainbow::{InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological};

use crate::Apply;

pub trait Collision<Diff: Send, State>: Send + Sized {
    type Output: Send;
    fn always_okay(diff: &Diff) -> bool;
    fn okay(self) -> Self::Output;
    fn check(self) -> object_rainbow::Result<Self::Output>;
}

pub struct ExposedState;
pub struct ConcealedState;

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    PartialEq,
    Eq,
    Default,
)]
pub struct NoOverwrites<T>(pub T);

impl<D: Send, T: Apply<D, Output = X>, X: Collision<D, ExposedState, Output = O>, O: Send> Apply<D>
    for NoOverwrites<T>
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

/// This is separate from [`NoOverwrites`] both because of potential implementation conflicts and
/// because of some semantical differences.
#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    PartialEq,
    Eq,
    Default,
)]
pub struct NoCollisions<T>(T);

impl<D: Send, T: Apply<D, Output = X>, X: Collision<D, ConcealedState, Output = O>, O: Send>
    Apply<D> for NoCollisions<T>
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

impl<K: Send, V: Send, S> Collision<(V, K), S> for Option<V> {
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
            Err(object_rainbow::Error::consistency(NotUnique))
        }
    }
}

impl<K: Send, V: Send> Collision<(Option<V>, K), ExposedState> for Option<V> {
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
            Err(object_rainbow::Error::consistency(NotUnique))
        }
    }
}

impl<T: Send> Collision<T, ConcealedState> for bool {
    type Output = ();

    fn always_okay(_: &T) -> bool {
        false
    }

    fn okay(self) -> Self::Output {
        assert!(self);
    }

    fn check(self) -> object_rainbow::Result<Self::Output> {
        if self {
            Ok(())
        } else {
            Err(object_rainbow::Error::consistency(NotUnique))
        }
    }
}

impl<T: Send> Collision<(bool, T), ExposedState> for bool {
    type Output = bool;

    fn always_okay((remove, _): &(bool, T)) -> bool {
        *remove
    }

    fn okay(self) -> Self::Output {
        self
    }

    fn check(self) -> object_rainbow::Result<Self::Output> {
        if self {
            Ok(true)
        } else {
            Err(object_rainbow::Error::consistency(NotUnique))
        }
    }
}
