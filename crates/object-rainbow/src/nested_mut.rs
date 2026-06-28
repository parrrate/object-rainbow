use futures_channel::oneshot;

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

impl<'a, T: Clone> RemoteMut<'a, T> {
    pub fn new(local: &'a mut T, remote: oneshot::Sender<oneshot::Sender<T>>) -> Self {
        let (return_to, borrowed) = oneshot::channel();
        remote.send(return_to).ok();
        Self { local, borrowed }
    }
}

pub struct NestedMut<T, F> {
    value: Option<T>,
    return_to: Option<oneshot::Sender<T>>,
    _future: F,
}

impl<T, F> Drop for NestedMut<T, F> {
    fn drop(&mut self) {
        self.return_to
            .take()
            .expect("invalid state")
            .send(self.value.take().expect("invalid state"))
            .ok();
    }
}

impl<T, F> NestedMut<T, F> {
    pub fn new(value: T, return_to: oneshot::Sender<T>, future: F) -> Self {
        Self {
            value: Some(value),
            return_to: Some(return_to),
            _future: future,
        }
    }
}
