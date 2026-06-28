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
