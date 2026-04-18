use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological, derive_for_wrapped,
};

use crate::Apply;

#[derive_for_wrapped]
pub trait MapToSet<K: Send, V: Send>: Send + Sync {
    type T: Send;
    fn map(&self, key: K, value: V)
    -> impl Send + Future<Output = object_rainbow::Result<Self::T>>;
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
pub struct MappedToSet<M>(M);

impl<K: Send + Clone, V: Send, M: MapToSet<K, V>> Apply<(Option<V>, (Option<V>, K))>
    for MappedToSet<M>
{
    type Output = Vec<(bool, M::T)>;

    fn apply(
        &mut self,
        (old, (new, key)): (Option<V>, (Option<V>, K)),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Output>> {
        async move {
            let mut diff = Vec::new();
            if let Some(value) = old {
                diff.push((true, self.0.map(key.clone(), value).await?));
            }
            if let Some(value) = new {
                diff.push((false, self.0.map(key, value).await?));
            }
            Ok(diff)
        }
    }
}

pub trait Collision<Diff: Send, State>: Send + Sized {
    type Output: Send;
    fn always_okay(diff: &Diff) -> bool;
    fn okay(self) -> Self::Output;
    fn check(self) -> object_rainbow::Result<Self::Output>;
}

pub struct ExposedState;

pub struct KeepUnique<T>(pub T);

impl<D: Send, T: Apply<D, Output = X>, X: Collision<D, ExposedState, Output = O>, O: Send> Apply<D>
    for KeepUnique<T>
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
