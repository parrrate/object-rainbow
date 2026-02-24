use std::{marker::PhantomData, sync::Arc};

use object_rainbow::{Fetch, Inline, Object};
use object_rainbow_history::{Apply, History};
use object_rainbow_store::{RainbowStoreMut, StoreMut};

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

impl<
    T: Inline<Extra> + Clone + Default + Apply<D>,
    D: Object<Extra> + Clone,
    S: RainbowStoreMut,
    Extra: 'static + Send + Sync + Clone,
> HistoryStore<T, D, S, Extra>
{
    pub async fn commit(&self, diff: D) -> object_rainbow::Result<()> {
        let mut history = self
            .store
            .load_or_init::<History<T, D>, _>(self.key.as_ref())
            .await?;
        history.fetch_mut().await?.commit(diff).await?;
        history.save().await?;
        Ok(())
    }

    pub async fn load(&self) -> object_rainbow::Result<T> {
        self.history().await?.tree().await
    }

    pub async fn history(&self) -> object_rainbow::Result<History<T, D>> {
        self.store
            .load_or_init(self.key.as_ref())
            .await?
            .fetch()
            .await
    }

    pub async fn forward(&self, other: History<T, D>) -> object_rainbow::Result<()> {
        let mut history = self
            .store
            .load_or_init::<History<T, D>, _>(self.key.as_ref())
            .await?;
        history.fetch_mut().await?.forward(other).await?;
        history.save().await?;
        Ok(())
    }
}
