use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures_channel::oneshot;

use crate::FailFuture;

struct NestedGuard<'a, T> {
    original: &'a mut T,
    returned: oneshot::Receiver<T>,
}

impl<T> Drop for NestedGuard<'_, T> {
    fn drop(&mut self) {
        if let Ok(Some(returned)) = self.returned.try_recv() {
            *self.original = returned;
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

pub struct Borrower<T>(oneshot::Sender<Lent<T>>);

impl<'a, T: Clone> NestedGuard<'a, T> {
    fn new(original: &'a mut T, borrower: Borrower<T>) -> Self {
        let (return_to, returned) = oneshot::channel();
        borrower
            .0
            .send(Lent {
                value: original.clone(),
                return_to,
            })
            .ok();
        Self { original, returned }
    }
}

pub trait LendTo: Clone {
    fn lend_to<T>(&mut self, borrower: Borrower<Self>) -> impl Future<Output = T> {
        async move {
            let _guard = NestedGuard::new(self, borrower);
            std::future::pending().await
        }
    }
}

impl<T: Clone> LendTo for T {}

pub struct NestedMut<'a, T> {
    lent: Option<Lent<T>>,
    _guard: oneshot::Receiver<FailFuture<'a, ()>>,
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

struct WaitingLease<'a, T> {
    borrowing: oneshot::Receiver<Lent<T>>,
    future: Option<FailFuture<'a, ()>>,
}

impl<'a, T> Future for WaitingLease<'a, T> {
    type Output = object_rainbow::Result<Option<NestedMut<'a, T>>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this
            .future
            .as_mut()
            .expect("invalid state")
            .as_mut()
            .poll(cx)?
            .is_ready()
        {
            Poll::Ready(Ok(None))
        } else {
            let Ok(lent) = ready!(Pin::new(&mut this.borrowing).poll(cx)) else {
                return Poll::Ready(Ok(None));
            };
            Poll::Ready(Ok(Some(NestedMut::new(
                lent,
                this.future.take().expect("invalid state"),
            ))))
        }
    }
}

impl<'a, T> NestedMut<'a, T> {
    fn new(lent: Lent<T>, future: FailFuture<'a, ()>) -> Self {
        let (send, recv) = oneshot::channel();
        send.send(future).ok();
        Self {
            lent: Some(lent),
            _guard: recv,
        }
    }

    pub async fn from_fn<F: 'a + Send + Future<Output = object_rainbow::Result<()>>>(
        f: impl FnOnce(Borrower<T>) -> F,
    ) -> object_rainbow::Result<Option<Self>> {
        let (lending, borrowing) = oneshot::channel();
        let future = Box::pin(f(Borrower(lending)));
        WaitingLease {
            borrowing,
            future: Some(future),
        }
        .await
    }
}
