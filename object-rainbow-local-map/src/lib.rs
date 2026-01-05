use std::sync::Arc;

use imbl::{HashMap, OrdSet};
use object_rainbow::{Hash, ObjectHashes, ToOutput};

struct EntryInner {
    topology: Vec<Hash>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct Entry {
    inner: Arc<EntryInner>,
    referenced_by: OrdSet<Hash>,
}

#[derive(Clone)]
pub struct LocalMap {
    map: HashMap<Hash, Entry>,
}

impl LocalMap {
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

    pub fn remove(&mut self, hash: Hash) -> Result<(), impl Iterator<Item = Hash>> {
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
}
