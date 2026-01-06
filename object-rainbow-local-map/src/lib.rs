use std::sync::Arc;

use imbl::{HashMap, OrdSet};
use object_rainbow::{Address, ByteNode, FailFuture, Hash, ObjectHashes, Resolve, ToOutput};

struct EntryInner {
    topology: Vec<Hash>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct Entry {
    inner: Arc<EntryInner>,
    referenced_by: OrdSet<Hash>,
}

#[derive(Clone, Default)]
pub struct LocalMap {
    map: HashMap<Hash, Entry>,
}

impl LocalMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        hash: Hash,
        tags_hash: Hash,
        topology: Vec<Hash>,
        data: Vec<u8>,
    ) -> object_rainbow::Result<()> {
        if self.contains(hash) {
            return Ok(());
        }
        let mut map = self.map.clone();
        let expected = ObjectHashes {
            tags: tags_hash,
            topology: topology.data_hash(),
            data: data.data_hash(),
        }
        .data_hash();
        if expected != hash {
            return Err(object_rainbow::Error::DataMismatch);
        }
        for referenced in &topology {
            map.get_mut(referenced)
                .ok_or(object_rainbow::Error::HashNotFound)?
                .referenced_by
                .insert(hash);
        }
        let inner = Arc::new(EntryInner { topology, data });
        let entry = Entry {
            inner,
            referenced_by: Default::default(),
        };
        map.insert(hash, entry);
        self.map = map;
        Ok(())
    }

    pub fn referenced_by(&self, hash: Hash) -> Option<impl use<> + Iterator<Item = Hash>> {
        if let Some(entry) = self.map.get(&hash)
            && !entry.referenced_by.is_empty()
        {
            Some(entry.referenced_by.clone().into_iter())
        } else {
            None
        }
    }

    pub fn remove(&mut self, hash: Hash) -> Result<(), impl use<> + Iterator<Item = Hash>> {
        if let Some(referenced_by) = self.referenced_by(hash) {
            return Err(referenced_by);
        }
        let mut map = self.map.clone();
        if let Some(Entry { inner, .. }) = self.map.remove(&hash) {
            for referenced in &inner.topology {
                map.get_mut(referenced)
                    .expect("unknown")
                    .referenced_by
                    .remove(&hash);
            }
        }
        self.map = map;
        Ok(())
    }

    pub fn get(&self, hash: Hash) -> Option<(&[Hash], &[u8])> {
        self.map
            .get(&hash)
            .map(|entry| (&*entry.inner.topology, &*entry.inner.data))
    }

    pub fn contains(&self, hash: Hash) -> bool {
        self.map.contains_key(&hash)
    }

    pub fn to_resolve(&self) -> Arc<dyn Resolve> {
        Arc::new(self.clone())
    }

    async fn resolve_bytes(&self, address: Address) -> object_rainbow::Result<Vec<u8>> {
        self.get(address.hash)
            .map(|(_, data)| data.to_owned())
            .ok_or(object_rainbow::Error::HashNotFound)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Resolve for LocalMap {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        Box::pin(async move {
            let data = self.resolve_bytes(address).await?;
            Ok((data, self.to_resolve()))
        })
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(self.resolve_bytes(address))
    }

    fn name(&self) -> &str {
        "local map"
    }
}
