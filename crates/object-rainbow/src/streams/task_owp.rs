use std::{
    ops::Bound,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

use atomic_waker::AtomicWaker;
use flume::{Receiver, Sender, r#async::RecvStream, bounded};
use futures_core::Stream;
use pin_project::pin_project;

use super::owp::{Owp, OwpBoundsOwned};

#[derive(Default)]
struct Wakers {
    min: AtomicWaker,
    max: AtomicWaker,
}

impl Wake for Wakers {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.min.wake();
        self.max.wake();
    }
}

#[pin_project]
pub struct TaskOwp<'a, T, B, F> {
    #[pin]
    future: Option<F>,
    #[pin]
    ready_min: RecvStream<'a, T>,
    #[pin]
    ready_max: RecvStream<'a, T>,
    bound_min: Sender<Bound<B>>,
    bound_max: Sender<Bound<B>>,
    retry_min: Receiver<Bound<B>>,
    retry_max: Receiver<Bound<B>>,
    wants_min: Arc<AtomicBool>,
    wants_max: Arc<AtomicBool>,
    update: Sender<()>,
    wakers: Arc<Wakers>,
    waker: Waker,
}

impl<'a, T, B, F: Future<Output = crate::Result<()>>> TaskOwp<'a, T, B, F> {
    pub fn new(f: impl FnOnce(OwpCtx<T, B>) -> F) -> Self {
        let (ready_min_s, ready_min_r) = bounded(1);
        let (ready_max_s, ready_max_r) = bounded(1);
        let (bound_min_s, bound_min_r) = bounded(1);
        let (bound_max_s, bound_max_r) = bounded(1);
        let (update_s, update_r) = bounded(1);
        let wants_min = Arc::new(AtomicBool::new(false));
        let wants_max = Arc::new(AtomicBool::new(false));
        let future = Some(f(OwpCtx {
            ready_min: ready_min_s,
            ready_max: ready_max_s,
            bound_min: bound_min_r.clone(),
            bound_max: bound_max_r.clone(),
            wants_min: wants_min.clone(),
            wants_max: wants_max.clone(),
            update: update_r,
            hook: update_s.clone(),
        }));
        let wakers = Arc::<Wakers>::default();
        let waker = wakers.clone().into();
        Self {
            future,
            ready_min: ready_min_r.into_stream(),
            ready_max: ready_max_r.into_stream(),
            bound_min: bound_min_s,
            bound_max: bound_max_s,
            retry_min: bound_min_r,
            retry_max: bound_max_r,
            wants_min,
            wants_max,
            update: update_s,
            wakers,
            waker,
        }
    }
}

impl<'a, T, B, F: Future<Output = crate::Result<()>>> Owp for TaskOwp<'a, T, B, F> {
    type Item = T;

    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        if let Poll::Ready(item) = this.ready_min.as_mut().poll_next(cx) {
            return Poll::Ready(Ok(item));
        }
        if let Some(future) = this.future.as_mut().as_pin_mut() {
            this.wants_min.store(true, Ordering::Relaxed);
            this.update.try_send(()).ok();
            this.wakers.min.register(cx.waker());
            if future
                .poll(&mut Context::from_waker(this.waker))?
                .is_ready()
            {
                this.future.set(None);
            }
            this.ready_min.poll_next(cx).map(Ok)
        } else {
            Poll::Ready(Ok(None))
        }
    }

    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let mut this = self.project();
        if let Poll::Ready(item) = this.ready_max.as_mut().poll_next(cx) {
            return Poll::Ready(Ok(item));
        }
        if let Some(future) = this.future.as_mut().as_pin_mut() {
            this.wants_max.store(true, Ordering::Relaxed);
            this.update.try_send(()).ok();
            this.wakers.max.register(cx.waker());
            if future
                .poll(&mut Context::from_waker(this.waker))?
                .is_ready()
            {
                this.future.set(None);
            }
            this.ready_max.poll_next(cx).map(Ok)
        } else {
            Poll::Ready(Ok(None))
        }
    }
}

fn intersect<B>(x: Bound<B>, y: Bound<B>, stricter: impl Fn(&B, &B) -> bool) -> Bound<B> {
    match (x, y) {
        (Bound::Unbounded, strict) => strict,
        (strict, Bound::Unbounded) => strict,
        (Bound::Included(strict), Bound::Included(other) | Bound::Excluded(other))
            if stricter(&strict, &other) =>
        {
            Bound::Included(strict)
        }
        (Bound::Excluded(strict), Bound::Included(other) | Bound::Excluded(other))
            if stricter(&strict, &other) =>
        {
            Bound::Excluded(strict)
        }
        (Bound::Included(other) | Bound::Excluded(other), Bound::Included(strict))
            if stricter(&strict, &other) =>
        {
            Bound::Included(strict)
        }
        (Bound::Included(other) | Bound::Excluded(other), Bound::Excluded(strict))
            if stricter(&strict, &other) =>
        {
            Bound::Excluded(strict)
        }
        (Bound::Excluded(strict), _) => Bound::Excluded(strict),
        (_, Bound::Excluded(strict)) => Bound::Excluded(strict),
        (Bound::Included(strict), Bound::Included(_)) => Bound::Included(strict),
    }
}

impl<'a, T, B: Ord, F> OwpBoundsOwned<B> for TaskOwp<'a, T, B, F> {
    fn set_min(self: Pin<&mut Self>, mut min: Bound<B>) {
        if let Ok(old) = self.retry_min.try_recv() {
            min = intersect(old, min, |strict, other| strict > other);
        }
        self.bound_min.try_send(min).ok();
        self.update.try_send(()).ok();
        self.wakers.wake_by_ref();
    }

    fn set_max(self: Pin<&mut Self>, mut max: Bound<B>) {
        if let Ok(old) = self.retry_max.try_recv() {
            max = intersect(old, max, |strict, other| strict < other);
        }
        self.bound_max.try_send(max).ok();
        self.update.try_send(()).ok();
        self.wakers.wake_by_ref();
    }
}

pub struct OwpCtx<T, B> {
    ready_min: Sender<T>,
    ready_max: Sender<T>,
    bound_min: Receiver<Bound<B>>,
    bound_max: Receiver<Bound<B>>,
    wants_min: Arc<AtomicBool>,
    wants_max: Arc<AtomicBool>,
    update: Receiver<()>,
    hook: Sender<()>,
}

#[derive(Clone)]
pub struct OwnHook {
    hook: Sender<()>,
}

impl OwnHook {
    pub fn notify(&self) {
        self.hook.try_send(()).ok();
    }
}

impl<T, B> OwpCtx<T, B> {
    pub fn wants_min(&mut self) -> bool {
        self.wants_min.load(Ordering::Relaxed)
    }

    pub fn wants_max(&mut self) -> bool {
        self.wants_max.load(Ordering::Relaxed)
    }

    pub fn give_min(&mut self, min: T) {
        assert!(self.wants_min(), "unexpected min");
        self.wants_min.store(false, Ordering::Relaxed);
        self.ready_min.try_send(min).ok();
    }

    pub fn give_max(&mut self, max: T) {
        assert!(self.wants_max(), "unexpected max");
        self.wants_max.store(false, Ordering::Relaxed);
        self.ready_max.try_send(max).ok();
    }

    pub fn bound_min(&mut self) -> Option<Bound<B>> {
        self.bound_min.try_recv().ok()
    }

    pub fn bound_max(&mut self) -> Option<Bound<B>> {
        self.bound_max.try_recv().ok()
    }

    pub async fn wait(&mut self) -> bool {
        self.update.recv_async().await.is_ok()
    }

    pub fn hook(&self) -> OwnHook {
        OwnHook {
            hook: self.hook.clone(),
        }
    }
}
