use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

use object_rainbow::{
    Address, Hash, Object, ObjectHashes, OptionalHash, PointVisitor, Resolve, Singular,
    SingularFetch, Topological, Traversible,
};
use object_rainbow_point::Point;

pub trait RainbowFuture: Send + Future<Output = object_rainbow::Result<Self::T>> {
    type T;
}

impl<F: Send + Future<Output = object_rainbow::Result<T>>, T> RainbowFuture for F {
    type T = T;
}

struct StoreVisitor<'a, 'x, S: ?Sized> {
    store: &'a S,
    futures: &'x mut Vec<Pin<Box<dyn 'a + Send + Future<Output = object_rainbow::Result<()>>>>>,
}

impl<'a, 'x, S: RainbowStore> PointVisitor for StoreVisitor<'a, 'x, S> {
    fn visit<T: Traversible>(&mut self, point: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let point = point.clone();
        let store = self.store;
        self.futures.push(Box::pin(async move {
            store.save_point(&point).await.map(|_| ())
        }));
    }
}

struct StoreResolve<S> {
    store: S,
}

impl<S: 'static + Send + RainbowStore> Resolve for StoreResolve<S> {
    fn resolve(
        &'_ self,
        address: Address,
    ) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        Box::pin(async move {
            let bytes = self.store.fetch(address.hash).await?.as_ref().to_vec();
            Ok((bytes, self.store.resolve()))
        })
    }

    fn resolve_data(&'_ self, address: Address) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let bytes = self.store.fetch(address.hash).await?.as_ref().to_vec();
            Ok(bytes)
        })
    }
}

pub trait RainbowStore: 'static + Send + Sync + Clone {
    fn saved_point<T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
        &self,
        point: &Point<T>,
        extra: Extra,
    ) -> impl RainbowFuture<T = Point<T>> {
        async {
            self.save_point(point).await?;
            Ok(point.with_resolve(self.resolve(), extra))
        }
    }
    fn save_point(&self, point: &impl SingularFetch<T: Traversible>) -> impl RainbowFuture<T = ()> {
        async {
            if !self.contains(point.hash()).await? {
                self.save_object(&point.fetch().await?).await?;
            }
            Ok(())
        }
    }
    fn save_topology(&self, object: &impl Topological) -> impl RainbowFuture<T = ()> {
        let mut futures = Vec::with_capacity(object.point_count());
        object.traverse(&mut StoreVisitor {
            store: self,
            futures: &mut futures,
        });
        async {
            for future in futures {
                future.await?;
            }
            Ok(())
        }
    }
    fn save_object(&self, object: &impl Traversible) -> impl RainbowFuture<T = ()> {
        async {
            self.save_topology(object).await?;
            self.save_data(object.hashes(), &object.vec()).await?;
            Ok(())
        }
    }
    fn resolve(&self) -> Arc<dyn Resolve> {
        Arc::new(StoreResolve {
            store: self.clone(),
        })
    }
    fn point_extra<T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
        &self,
        hash: Hash,
        extra: Extra,
    ) -> Point<T> {
        Point::from_address_extra(Address::from_hash(hash), self.resolve(), extra)
    }
    fn point<T: Object>(&self, hash: Hash) -> Point<T> {
        self.point_extra(hash, ())
    }
    fn save_data(&self, hashes: ObjectHashes, data: &[u8]) -> impl RainbowFuture<T = ()>;
    fn contains(&self, hash: Hash) -> impl RainbowFuture<T = bool>;
    fn fetch(&self, hash: Hash)
    -> impl RainbowFuture<T = impl 'static + Send + Sync + AsRef<[u8]>>;
}

pub trait RainbowStoreMut: RainbowStore {
    fn create_ref(
        &self,
        hash: Hash,
    ) -> impl RainbowFuture<T = impl 'static + Send + Sync + AsRef<str>> {
        let _ = hash;
        async { Err::<String, _>(object_rainbow::Error::Unimplemented) }
    }
    fn update_ref(
        &self,
        key: &str,
        old: Option<OptionalHash>,
        hash: Hash,
    ) -> impl RainbowFuture<T = ()>;
    fn fetch_ref(&self, key: &str) -> impl RainbowFuture<T = OptionalHash>;
    fn ref_exists(&self, key: &str) -> impl RainbowFuture<T = bool>;
    fn store_ref_raw<
        T: Object<Extra>,
        K: Send + Sync + AsRef<str>,
        Extra: 'static + Send + Sync + Clone,
    >(
        &self,
        key: K,
        point: Point<T>,
        extra: Extra,
    ) -> StoreRef<Self, K, T, Extra> {
        StoreRef {
            store: self.clone(),
            key,
            old: point.hash().into(),
            point,
            extra,
        }
    }
}

#[derive(Clone)]
pub struct StoreMut<S, Extra = ()> {
    store: S,
    extra: Extra,
}

impl<S> StoreMut<S> {
    pub const fn new(store: S) -> Self {
        Self::new_extra(store, ())
    }
}

impl<S, Extra> StoreMut<S, Extra> {
    pub const fn new_extra(store: S, extra: Extra) -> Self {
        Self { store, extra }
    }
}

impl<S: RainbowStoreMut, Extra: 'static + Send + Sync + Clone> StoreMut<S, Extra> {
    pub async fn exists<K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
    ) -> object_rainbow::Result<bool> {
        self.store.ref_exists(key.as_ref()).await
    }

    pub async fn create<T: Object<Extra>>(
        &self,
        point: Point<T>,
    ) -> object_rainbow::Result<StoreRef<S, impl 'static + Send + Sync + AsRef<str>, T, Extra>>
    {
        let point = self.store.saved_point(&point, self.extra.clone()).await?;
        let key = self.store.create_ref(point.hash()).await?;
        Ok(self.store.store_ref_raw(key, point, self.extra.clone()))
    }

    pub async fn update<T: Object<Extra>, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
        point: Point<T>,
    ) -> object_rainbow::Result<StoreRef<S, K, T, Extra>> {
        let point = self.store.saved_point(&point, self.extra.clone()).await?;
        self.store
            .update_ref(key.as_ref(), None, point.hash())
            .await?;
        Ok(self.store.store_ref_raw(key, point, self.extra.clone()))
    }

    pub async fn init<T: Object<Extra>, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
        point: Point<T>,
    ) -> object_rainbow::Result<StoreRef<S, K, T, Extra>> {
        let point = self.store.saved_point(&point, self.extra.clone()).await?;
        self.store
            .update_ref(key.as_ref(), Some(OptionalHash::NONE), point.hash())
            .await?;
        Ok(self.store.store_ref_raw(key, point, self.extra.clone()))
    }

    pub async fn load<T: Object<Extra>, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
    ) -> object_rainbow::Result<StoreRef<S, K, T, Extra>> {
        let hash = self
            .store
            .fetch_ref(key.as_ref())
            .await?
            .get()
            .ok_or(object_rainbow::Error::HashNotFound)?;
        let point = self.store.point_extra(hash, self.extra.clone());
        Ok(self.store.store_ref_raw(key, point, self.extra.clone()))
    }

    pub async fn reference<T: Object<Extra>, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
        point: Point<T>,
    ) -> object_rainbow::Result<StoreRef<S, K, T, Extra>> {
        Ok(StoreRef {
            old: self.store.fetch_ref(key.as_ref()).await?,
            ..self.store.store_ref_raw(key, point, self.extra.clone())
        })
    }
}

pub struct StoreRef<S, K, T, Extra> {
    store: S,
    key: K,
    old: OptionalHash,
    point: Point<T>,
    extra: Extra,
}

impl<S, K, T, Extra> Deref for StoreRef<S, K, T, Extra> {
    type Target = Point<T>;

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}

impl<S, K, T, Extra> DerefMut for StoreRef<S, K, T, Extra> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.point
    }
}

impl<
    S: RainbowStoreMut,
    K: Send + Sync + AsRef<str>,
    T: Object<Extra>,
    Extra: 'static + Send + Sync + Clone,
> StoreRef<S, K, T, Extra>
{
    pub fn is_modified(&self) -> bool {
        self.point.hash() != self.old
    }

    pub fn is_new(&self) -> bool {
        self.old.is_none()
    }

    pub async fn save_point(&mut self) -> object_rainbow::Result<()> {
        self.point = self
            .store
            .saved_point(&self.point, self.extra.clone())
            .await?;
        Ok(())
    }

    pub async fn save(&mut self) -> object_rainbow::Result<()> {
        if self.is_modified() {
            self.save_point().await?;
            self.store
                .update_ref(self.key.as_ref(), Some(self.old), self.point.hash())
                .await?;
            self.old = self.point.hash().into();
        }
        Ok(())
    }
}
