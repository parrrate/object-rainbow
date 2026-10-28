use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    pin::Pin,
    task::{Context, Poll, ready},
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

    fn with_bounds<B: RangeBounds<T>, T: ?Sized>(self, bounds: B) -> WithBounds<Self, B, T>
    where
        Self: Sized + OwpBounds<T>,
    {
        WithBounds {
            owp: self,
            bounds: Some(bounds),
            _item: PhantomData,
        }
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

#[pin_project]
pub struct WithBounds<O, B, T: ?Sized> {
    #[pin]
    owp: O,
    bounds: Option<B>,
    _item: PhantomData<T>,
}

impl<O, B, T: ?Sized> WithBounds<O, B, T> {
    fn inner(self: Pin<&mut Self>) -> Pin<&mut O> {
        self.project().owp
    }
}

impl<O: OwpBounds<T>, B: RangeBounds<T>, T: ?Sized> WithBounds<O, B, T> {
    fn init(self: Pin<&mut Self>) {
        let mut this = self.project();
        if let Some(bounds) = this.bounds.take() {
            this.owp.as_mut().set_min(bounds.start_bound());
            this.owp.as_mut().set_max(bounds.end_bound());
        }
    }
}

impl<O: Owp + OwpBounds<T>, B: RangeBounds<T>, T: ?Sized> Owp for WithBounds<O, B, T> {
    type Item = O::Item;

    fn poll_min(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        self.as_mut().init();
        self.inner().poll_min(cx)
    }

    fn poll_max(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        self.as_mut().init();
        self.inner().poll_max(cx)
    }
}

#[pin_project]
pub struct Distinct<O> {
    #[pin]
    owp: O,
}

impl<O: Owp<Item = (K, V)> + OwpBounds<K>, K, V> Distinct<O> {
    pub fn new(owp: O) -> Self {
        Self { owp }
    }
}

impl<O: Owp<Item = (K, V)> + OwpBounds<K>, K, V> Owp for Distinct<O> {
    type Item = (K, V);

    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        Poll::Ready(Ok(
            if let Some((k, v)) = ready!(this.owp.as_mut().poll_min(cx))? {
                this.owp.set_min(Bound::Excluded(&k));
                Some((k, v))
            } else {
                None
            },
        ))
    }

    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        Poll::Ready(Ok(
            if let Some((k, v)) = ready!(this.owp.as_mut().poll_min(cx))? {
                this.owp.set_max(Bound::Excluded(&k));
                Some((k, v))
            } else {
                None
            },
        ))
    }
}
