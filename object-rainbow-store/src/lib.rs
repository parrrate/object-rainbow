use std::pin::Pin;

use object_rainbow::{
    Fetch, Hash, Object, ObjectHashes, Point, PointVisitor, Singular, ToOutputExt, Topological,
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

impl<'a, 'x, S: ?Sized + RainbowStore, Extra: 'static + Send + Sync> PointVisitor<Extra>
    for StoreVisitor<'a, 'x, S>
{
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        let point = point.clone();
        let store = self.store;
        self.futures
            .push(Box::pin(async move { store.save_point(&point).await }));
    }
}

pub trait RainbowStore: Sync {
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
    fn save_data(&self, hashes: ObjectHashes, data: &[u8]) -> impl RainbowFuture<T = ()>;
    fn contains(&self, hash: Hash) -> impl RainbowFuture<T = bool>;
    fn fetch(&self, hash: Hash) -> impl RainbowFuture<T: Send + Sync + AsRef<[u8]>>;
}
