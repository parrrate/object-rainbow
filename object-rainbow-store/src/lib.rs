use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

use object_rainbow::{
    Address, Fetch, Hash, Object, ObjectHashes, OptionalHash, Point, PointVisitor, Resolve,
    Singular, Topological, Traversible,
};

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
    fn visit<T: Traversible>(&mut self, point: &Point<T>) {
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

    fn name(&self) -> &str {
        self.store.name()
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
    fn save_point<T: Traversible>(&self, point: &Point<T>) -> impl RainbowFuture<T = ()> {
        async {
            if !self.contains(point.hash()).await? {
                self.save_object(&point.fetch().await?).await?;
            }
            Ok(())
        }
    }
    fn save_topology(&self, object: &impl Topological) -> impl RainbowFuture<T = ()> {
        let mut futures = Vec::with_capacity(object.point_count());
        object.accept_points(&mut StoreVisitor {
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
    fn name(&self) -> &str;
}

pub trait RainbowStoreMut: RainbowStore {
    fn create_ref(
        &self,
        hash: Hash,
    ) -> impl RainbowFuture<T = impl 'static + Send + Sync + AsRef<str>> {
        let _ = hash;
        async { Err::<String, _>(object_rainbow::error_fetch!("not supported")) }
    }
    fn update_ref(
        &self,
        key: &str,
        old: Option<OptionalHash>,
        hash: Hash,
    ) -> impl RainbowFuture<T = ()>;
    fn fetch_ref(&self, key: &str) -> impl RainbowFuture<T = OptionalHash>;
    fn ref_exists(&self, key: &str) -> impl RainbowFuture<T = bool>;
    fn create<T: Object>(
        &self,
        point: Point<T>,
    ) -> impl RainbowFuture<T = StoreRef<Self, impl 'static + Send + Sync + AsRef<str>, T, ()>>
    {
        async move {
            let point = self.saved_point(&point, ()).await?;
            let key = self.create_ref(point.hash()).await?;
            Ok(self.store_ref_raw(key, point, ()))
        }
    }
    fn update<T: Object, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
        point: Point<T>,
    ) -> impl RainbowFuture<T = StoreRef<Self, K, T, ()>> {
        async move {
            let point = self.saved_point(&point, ()).await?;
            self.update_ref(key.as_ref(), None, point.hash()).await?;
            Ok(self.store_ref_raw(key, point, ()))
        }
    }
    fn load<T: Object, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
    ) -> impl RainbowFuture<T = StoreRef<Self, K, T, ()>> {
        async move {
            let hash = self
                .fetch_ref(key.as_ref())
                .await?
                .get()
                .ok_or_else(|| object_rainbow::error_fetch!("key not found"))?;
            let point = self.point(hash);
            Ok(self.store_ref_raw(key, point, ()))
        }
    }
    fn reference<T: Object, K: Send + Sync + AsRef<str>>(
        &self,
        key: K,
        point: Point<T>,
    ) -> impl RainbowFuture<T = StoreRef<Self, K, T, ()>> {
        async move {
            Ok(StoreRef {
                old: self.fetch_ref(key.as_ref()).await?,
                ..self.store_ref_raw(key, point, ())
            })
        }
    }
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
