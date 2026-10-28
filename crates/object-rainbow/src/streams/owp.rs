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
    fn set_min(self: Pin<&mut Self>, min: Bound<&Self::Item>);
    fn set_max(self: Pin<&mut Self>, max: Bound<&Self::Item>);
}
