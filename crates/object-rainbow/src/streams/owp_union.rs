use std::{
    collections::{BTreeMap, VecDeque},
    ops::Bound,
    pin::Pin,
    task::{Context, Poll},
};

use ruchei_wakelist::{Queue, Ref};

use super::owp::{Owp, OwpBounds};

const OP_WAKE_MIN: usize = 0;
const OP_WAKE_MAX: usize = 1;
const OP_WAKE: usize = 2;
const OP_IS_WAITING_MIN: usize = 2;
const OP_IS_WAITING_MAX: usize = 3;
const OP_COUNT: usize = 4;

pub struct OwpUnion<O, K, V> {
    queue: Queue<O, OP_WAKE, OP_COUNT>,
    pulled: BTreeMap<K, VecDeque<(V, Ref<O, OP_WAKE, OP_COUNT>)>>,
}

impl<O, K, V> Unpin for OwpUnion<O, K, V> {}

impl<O: Owp<Item = (K, V)>, K: Ord + Clone, V> OwpUnion<O, K, V> {
    pub fn new(owp: impl IntoIterator<Item = O>) -> Self {
        let mut queue = Queue::new();
        for owp in owp {
            let r = queue.insert(owp);
            queue.link_push_back::<OP_WAKE_MIN>(&r);
            queue.link_push_back::<OP_WAKE_MAX>(&r);
            queue.link_push_back::<OP_IS_WAITING_MIN>(&r);
            queue.link_push_back::<OP_IS_WAITING_MAX>(&r);
        }
        Self {
            queue,
            pulled: Default::default(),
        }
    }
}

impl<O: Owp<Item = (K, V)>, K: Ord + Clone, V> Owp for OwpUnion<O, K, V> {
    type Item = (K, V);

    fn poll_min(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let this = self.get_mut();
        this.queue.queue_poll::<OP_WAKE_MIN>(cx);
        while let Some(r) = this.queue.link_pop_front::<OP_WAKE_MIN>() {
            if !this.queue.link_pop_at::<OP_IS_WAITING_MIN>(&r) {
                continue;
            }
            let (o, waker) = this.queue.context::<OP_WAKE_MIN>(&r);
            match o.poll_min(&mut Context::from_waker(&waker))? {
                Poll::Ready(Some((k, v))) => {
                    this.pulled.entry(k).or_default().push_back((v, r.clone()))
                }
                Poll::Ready(None) => assert!(this.queue.remove_pinned(&r)),
                Poll::Pending => assert!(this.queue.link_push_back::<OP_IS_WAITING_MIN>(&r)),
            }
        }
        if !this.queue.link_empty::<OP_IS_WAITING_MIN>() {
            Poll::Pending
        } else if let Some(mut min) = this.pulled.first_entry() {
            let (v, r) = min.get_mut().pop_front().unwrap();
            let k = if min.get().is_empty() {
                min.remove_entry().0
            } else {
                min.key().clone()
            };
            this.queue.link_push_back::<OP_WAKE_MIN>(&r);
            this.queue.link_push_back::<OP_IS_WAITING_MIN>(&r);
            Poll::Ready(Ok(Some((k, v))))
        } else {
            Poll::Ready(Ok(None))
        }
    }

    fn poll_max(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<Option<Self::Item>>> {
        let this = self.get_mut();
        this.queue.queue_poll::<OP_WAKE_MAX>(cx);
        while let Some(r) = this.queue.link_pop_front::<OP_WAKE_MAX>() {
            if !this.queue.link_pop_at::<OP_IS_WAITING_MAX>(&r) {
                continue;
            }
            let (o, waker) = this.queue.context::<OP_WAKE_MAX>(&r);
            match o.poll_max(&mut Context::from_waker(&waker))? {
                Poll::Ready(Some((k, v))) => {
                    this.pulled.entry(k).or_default().push_back((v, r.clone()))
                }
                Poll::Ready(None) => assert!(this.queue.remove_pinned(&r)),
                Poll::Pending => assert!(this.queue.link_push_back::<OP_IS_WAITING_MAX>(&r)),
            }
        }
        if !this.queue.link_empty::<OP_IS_WAITING_MAX>() {
            Poll::Pending
        } else if let Some(mut max) = this.pulled.last_entry() {
            let (v, r) = max.get_mut().pop_front().unwrap();
            let k = if max.get().is_empty() {
                max.remove_entry().0
            } else {
                max.key().clone()
            };
            this.queue.link_push_back::<OP_WAKE_MAX>(&r);
            this.queue.link_push_back::<OP_IS_WAITING_MAX>(&r);
            Poll::Ready(Ok(Some((k, v))))
        } else {
            Poll::Ready(Ok(None))
        }
    }
}

impl<O: OwpBounds<B>, K, V, B: ?Sized> OwpBounds<B> for OwpUnion<O, K, V> {
    fn set_min(self: Pin<&mut Self>, min: Bound<&B>) {
        let this = self.get_mut();
        this.queue.values_mut().for_each(|o| o.set_min(min));
        let mut r = None;
        while let (_, Some(next)) = this.queue.link_of::<OP_IS_WAITING_MIN>(r.as_ref()) {
            this.queue.link_push_back::<OP_WAKE_MIN>(&next);
            r = Some(next);
        }
    }

    fn set_max(self: Pin<&mut Self>, max: Bound<&B>) {
        let this = self.get_mut();
        this.queue.values_mut().for_each(|o| o.set_max(max));
        let mut r = None;
        while let (_, Some(next)) = this.queue.link_of::<OP_IS_WAITING_MAX>(r.as_ref()) {
            this.queue.link_push_back::<OP_WAKE_MAX>(&next);
            r = Some(next);
        }
    }
}
