use std::{marker::PhantomData, sync::Arc};

use object_rainbow_store::StoreMut;

pub struct HistoryStore<T, S, Extra = ()> {
    key: Arc<str>,
    store: StoreMut<S, Extra>,
    _tree: PhantomData<T>,
}

impl<T, S, Extra> HistoryStore<T, S, Extra> {
    pub fn new_extra(key: &str, store: S, extra: Extra) -> Self {
        Self {
            key: key.into(),
            store: StoreMut::new_extra(store, extra),
            _tree: PhantomData,
        }
    }
}

impl<T, S: Clone, Extra: Clone> Clone for HistoryStore<T, S, Extra> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            store: self.store.clone(),
            _tree: PhantomData,
        }
    }
}
