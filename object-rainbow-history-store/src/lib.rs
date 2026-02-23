use std::{marker::PhantomData, sync::Arc};

use object_rainbow_store::StoreMut;

pub struct HistoryStore<T, D, S, Extra = ()> {
    key: Arc<str>,
    store: StoreMut<S, Extra>,
    _marker: PhantomData<(T, D)>,
}

impl<T, D, S> HistoryStore<T, D, S> {
    pub fn new(key: &str, store: S) -> Self {
        Self::new_extra(key, store, ())
    }
}

impl<T, D, S, Extra> HistoryStore<T, D, S, Extra> {
    pub fn new_extra(key: &str, store: S, extra: Extra) -> Self {
        Self {
            key: key.into(),
            store: StoreMut::new_extra(store, extra),
            _marker: PhantomData,
        }
    }
}

impl<T, D, S: Clone, Extra: Clone> Clone for HistoryStore<T, D, S, Extra> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            store: self.store.clone(),
            _marker: PhantomData,
        }
    }
}
