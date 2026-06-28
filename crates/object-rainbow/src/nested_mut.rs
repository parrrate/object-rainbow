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
