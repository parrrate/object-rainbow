use std::{
    ops::Bound,
    pin::Pin,
    task::ready,
    task::{Context, Poll},
};

use pin_project::pin_project;

use crate::ToOutput;

pub trait Owp {
    type Item;
    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<object_rainbow::Result<Option<Self::Item>>>;
    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<object_rainbow::Result<Option<Self::Item>>>;

    fn try_for_each_increasing<F: FnMut(Self::Item)>(self, f: F) -> TryForEachIncreasing<Self, F>
    where
        Self: Sized,
    {
        TryForEachIncreasing { owp: self, f }
    }

    fn try_for_each_decreasing<F: FnMut(Self::Item)>(self, f: F) -> TryForEachDecreasing<Self, F>
    where
        Self: Sized,
    {
        TryForEachDecreasing { owp: self, f }
    }

    fn output_bounds(self) -> OutputBounds<Self>
    where
        Self: Sized,
    {
        OutputBounds { owp: self }
    }
}

pub trait OwpBounds<B: ?Sized = <Self as Owp>::Item> {
    fn set_min(self: Pin<&mut Self>, min: Bound<&B>);
    fn set_max(self: Pin<&mut Self>, max: Bound<&B>);
}

pub trait OwpBoundsOwned<B = <Self as Owp>::Item> {
    fn set_min(self: Pin<&mut Self>, min: Bound<B>);
    fn set_max(self: Pin<&mut Self>, max: Bound<B>);
}

#[pin_project]
pub struct TryForEachIncreasing<O, F> {
    #[pin]
    owp: O,
    f: F,
}

impl<O: Owp, F: FnMut(O::Item)> Future for TryForEachIncreasing<O, F> {
    type Output = object_rainbow::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        while let Some(item) = ready!(this.owp.as_mut().poll_min(cx))? {
            (this.f)(item);
        }
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
pub struct TryForEachDecreasing<O, F> {
    #[pin]
    owp: O,
    f: F,
}

impl<O: Owp, F: FnMut(O::Item)> Future for TryForEachDecreasing<O, F> {
    type Output = object_rainbow::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        while let Some(item) = ready!(this.owp.as_mut().poll_max(cx))? {
            (this.f)(item);
        }
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
pub struct OutputBounds<O> {
    #[pin]
    owp: O,
}

impl<O> OutputBounds<O> {
    fn inner(self: Pin<&mut Self>) -> Pin<&mut O> {
        self.project().owp
    }
}

impl<O: Owp> Owp for OutputBounds<O> {
    type Item = O::Item;

    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        self.inner().poll_min(cx)
    }

    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        self.inner().poll_max(cx)
    }
}

impl<O: Owp + OwpBoundsOwned<Vec<u8>>, T: ToOutput> OwpBounds<T> for OutputBounds<O> {
    fn set_min(self: Pin<&mut Self>, min: Bound<&T>) {
        self.inner().set_min(min.map(ToOutput::output));
    }

    fn set_max(self: Pin<&mut Self>, max: Bound<&T>) {
        self.inner().set_max(max.map(ToOutput::output));
    }
}
