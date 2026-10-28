use std::{
    ops::Bound,
    pin::Pin,
    task::{Context, Poll},
};

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
}

pub trait OwpBounds<B = <Self as Owp>::Item> {
    fn set_min(self: Pin<&mut Self>, min: Bound<&B>);
    fn set_max(self: Pin<&mut Self>, max: Bound<&B>);
}
