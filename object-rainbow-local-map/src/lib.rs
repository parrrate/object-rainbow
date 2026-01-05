use imbl::HashMap;
use object_rainbow::{Hash, ObjectHashes, ToOutput};

#[derive(Clone)]
struct Entry {
    topology: Vec<Hash>,
    data: Vec<u8>,
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
        self.map.insert(hash, Entry { topology, data });
        Ok(())
    }

    pub fn get(&self, hash: Hash) -> Option<(&[Hash], &[u8])> {
        self.map
            .get(&hash)
            .map(|Entry { topology, data }| (&**topology, &**data))
    }

    pub fn contains(&self, hash: Hash) -> bool {
        self.map.contains_key(&hash)
    }
}
