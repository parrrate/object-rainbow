use std::sync::Arc;

use imbl::HashMap;
use object_rainbow::{Hash, ObjectHashes, ToOutput};

struct EntryInner {
    topology: Vec<Hash>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct Entry {
    inner: Arc<EntryInner>,
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
        let expected = ObjectHashes {
            tags: tags_hash,
            topology: topology.data_hash(),
            data: data.data_hash(),
        }
        .data_hash();
        if expected != hash {
            return Err(object_rainbow::Error::DataMismatch);
        }
        let inner = Arc::new(EntryInner { topology, data });
        self.map.insert(hash, Entry { inner });
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
