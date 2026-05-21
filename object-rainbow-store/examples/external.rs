use std::sync::Arc;

use dashmap::DashMap;
use object_rainbow::{Fetch, Hash, ToOutput};
use object_rainbow_point::{IntoPoint, Point};
use object_rainbow_store::ExternalStore;

#[derive(Clone, Default)]
struct Store(DashMap<Hash, Vec<u8>>, Arc<()>);

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.1, &other.1)
    }
}

impl ExternalStore for Store {
    type Id = Hash;

    async fn save_data(
        &self,
        data: &[u8],
        _: &[Self::Id],
        _: Hash,
    ) -> object_rainbow::Result<Self::Id> {
        let id = data.data_hash();
        println!("save");
        self.0.insert(id, data.into());
        Ok(id)
    }

    async fn contains_data(
        &self,
        data: &[u8],
        _: &[Self::Id],
        _: Hash,
    ) -> object_rainbow::Result<bool> {
        self.contains(&data.data_hash()).await
    }

    async fn contains(&self, id: &Self::Id) -> object_rainbow::Result<bool> {
        Ok(self.0.contains_key(id))
    }

    async fn fetch(
        &self,
        id: &Self::Id,
    ) -> object_rainbow::Result<impl 'static + Send + Sync + AsRef<[u8]>> {
        self.0
            .get(id)
            .ok_or(object_rainbow::Error::HashNotFound)
            .map(|r| r.clone())
    }
}

fn main() -> object_rainbow::Result<()> {
    smol::block_on(async move {
        let store = &Store::default();
        let id = store
            .store_object((*b"a", (*b"b", (*b"c").point()).point()))
            .await?;
        #[expect(clippy::type_complexity)]
        let mut object: ([u8; 1], Point<([u8; 1], Point<[u8; 1]>)>) = store.load(&id).await?;
        assert_eq!(object.1.fetch().await?.1.fetch().await?, *b"c");
        println!("before mut");
        object.1.fetch_mut().await?.0 = *b"d";
        store.store_object(object).await?;
        Ok(())
    })
}
