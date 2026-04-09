use object_rainbow::{Hash, ObjectHashes, OptionalHash, ParseSliceRefless, ToOutput};
use object_rainbow_store::{RainbowStore, RainbowStoreMut};
use opendal::{ErrorKind, Operator};

#[derive(Debug, Clone)]
pub struct OpendalStore {
    operator: Operator,
}

impl OpendalStore {
    pub fn from_operator(operator: Operator) -> Self {
        Self { operator }
    }
}

impl RainbowStore for OpendalStore {
    async fn save_data(&self, hashes: ObjectHashes, data: &[u8]) -> object_rainbow::Result<()> {
        self.operator
            .write(&hex::encode(hashes.data_hash()), data.to_vec())
            .await
            .map_err(std::io::Error::from)?;
        Ok(())
    }

    async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.operator
            .exists(&hex::encode(hash))
            .await
            .map_err(std::io::Error::from)
            .map_err(object_rainbow::Error::from)
    }

    async fn fetch(
        &self,
        hash: Hash,
    ) -> object_rainbow::Result<impl 'static + Send + Sync + AsRef<[u8]>> {
        self.operator
            .read(&hex::encode(hash))
            .await
            .map_err(std::io::Error::from)
            .map_err(object_rainbow::Error::from)
            .map(|b| b.to_bytes())
    }

    fn name(&self) -> &str {
        "opendal"
    }
}

impl RainbowStoreMut for OpendalStore {
    async fn update_ref(
        &self,
        key: &str,
        _old: Option<OptionalHash>,
        hash: Hash,
    ) -> object_rainbow::Result<()> {
        self.operator
            .write(key, hash.to_vec())
            .await
            .map_err(std::io::Error::from)?;
        Ok(())
    }

    async fn fetch_ref(&self, key: &str) -> object_rainbow::Result<OptionalHash> {
        match self.operator.read(key).await {
            Ok(value) => OptionalHash::parse_slice_refless(&value.to_vec()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Default::default()),
            Err(e) => Err(object_rainbow::Error::from(std::io::Error::from(e))),
        }
    }

    async fn ref_exists(&self, key: &str) -> object_rainbow::Result<bool> {
        self.operator
            .exists(key)
            .await
            .map_err(std::io::Error::from)
            .map_err(object_rainbow::Error::from)
    }
}
