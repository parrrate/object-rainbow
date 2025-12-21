use std::{pin::Pin, sync::Arc};

use object_rainbow::{
    Address, Fetch, Hash, Object, ObjectHashes, Point, PointVisitor, Resolve, Singular,
    ToOutputExt, Topological,
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

impl<'a, 'x, S: RainbowStore, Extra: 'static + Send + Sync> PointVisitor<Extra>
    for StoreVisitor<'a, 'x, S>
{
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        let point = point.clone();
        let store = self.store;
        self.futures
            .push(Box::pin(async move { store.save_point(&point).await }));
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
            Ok((
                bytes,
                Arc::new(Self {
                    store: self.store.clone(),
                }) as _,
            ))
        })
    }

    fn name(&self) -> &str {
        self.store.name()
    }
}

pub trait RainbowStore: 'static + Send + Sync + Clone {
    fn save_point<Extra: 'static + Send + Sync>(
        &self,
        point: &Point<impl Object<Extra>, Extra>,
    ) -> impl RainbowFuture<T = ()> {
        async {
            if !self.contains(*point.hash()).await? {
                self.save_object(&point.fetch().await?).await?;
            }
            Ok(())
        }
    }
    fn save_topology<Extra: 'static + Send + Sync>(
        &self,
        object: &impl Topological<Extra>,
    ) -> impl RainbowFuture<T = ()> {
        let mut futures = Vec::new();
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
    fn save_object<Extra: 'static + Send + Sync>(
        &self,
        object: &impl Object<Extra>,
    ) -> impl RainbowFuture<T = ()> {
        async {
            self.save_topology(object).await?;
            self.save_data(object.hashes(), &object.output::<Vec<_>>())
                .await?;
            Ok(())
        }
    }
    fn point_extra<T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
        &self,
        address: Address,
        extra: Extra,
    ) -> Point<T, Extra> {
        Point::from_address_extra(
            address,
            Arc::new(StoreResolve {
                store: self.clone(),
            }),
            extra,
        )
    }
    fn save_data(&self, hashes: ObjectHashes, data: &[u8]) -> impl RainbowFuture<T = ()>;
    fn contains(&self, hash: Hash) -> impl RainbowFuture<T = bool>;
    fn fetch(&self, hash: Hash) -> impl RainbowFuture<T: Send + Sync + AsRef<[u8]>>;
    fn name(&self) -> &str;
}
