use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures_channel::oneshot;
use futures_util::{FutureExt, future::BoxFuture};

pub struct RemoteMut<'a, T> {
    local: &'a mut T,
    borrowed: oneshot::Receiver<T>,
}

impl<T> Drop for RemoteMut<'_, T> {
    fn drop(&mut self) {
        if let Ok(Some(returned)) = self.borrowed.try_recv() {
            *self.local = returned;
        }
    }
}

struct Lent<T> {
    value: T,
    return_to: oneshot::Sender<T>,
}

impl<T> Deref for Lent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Lent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Lent<T> {
    fn finish(self) {
        self.return_to.send(self.value).ok();
    }
}

pub struct Lender<T>(oneshot::Sender<Lent<T>>);

impl<'a, T: Clone> RemoteMut<'a, T> {
    pub fn new(local: &'a mut T, remote: Lender<T>) -> Self {
        let (return_to, borrowed) = oneshot::channel();
        remote
            .0
            .send(Lent {
                value: local.clone(),
                return_to,
            })
            .ok();
        Self { local, borrowed }
    }
}

pub struct NestedMut<'a, T> {
    lent: Option<Lent<T>>,
    _future: BoxFuture<'a, object_rainbow::Result<()>>,
}

impl<T> Deref for NestedMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lent.as_ref().expect("invalid state")
    }
}

impl<T> DerefMut for NestedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lent.as_mut().expect("invalid state")
    }
}

impl<T> Drop for NestedMut<'_, T> {
    fn drop(&mut self) {
        self.lent.take().expect("invalid state").finish();
    }
}

pub struct WaitingLease<'a, T> {
    borrowing: oneshot::Receiver<Lent<T>>,
    future: Option<BoxFuture<'a, object_rainbow::Result<()>>>,
}

impl<'a, T> Future for WaitingLease<'a, T> {
    type Output = object_rainbow::Result<Option<NestedMut<'a, T>>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this
            .future
            .as_mut()
            .expect("invalid state")
            .poll_unpin(cx)?
            .is_ready()
        {
            Poll::Ready(Ok(None))
        } else {
            let Ok(Lent { value, return_to }) = ready!(this.borrowing.poll_unpin(cx)) else {
                return Poll::Ready(Ok(None));
            };
            Poll::Ready(Ok(Some(NestedMut::new(
                value,
                return_to,
                this.future.take().expect("invalid state"),
            ))))
        }
    }
}

impl<'a, T> NestedMut<'a, T> {
    pub fn new(
        value: T,
        return_to: oneshot::Sender<T>,
        future: BoxFuture<'a, object_rainbow::Result<()>>,
    ) -> Self {
        Self {
            lent: Some(Lent { value, return_to }),
            _future: future,
        }
    }
}
