use object_rainbow::{Hash, ObjectHashes, ToOutput};
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
            .map_err(|e| object_rainbow::error_fetch!("{e}"))?;
        Ok(())
    }

    async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.operator
            .exists(&hex::encode(hash))
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))
    }

    async fn fetch(
        &self,
        hash: Hash,
    ) -> object_rainbow::Result<impl 'static + Send + Sync + AsRef<[u8]>> {
        self.operator
            .read(&hex::encode(hash))
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))
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
        _old: Option<Hash>,
        hash: Hash,
    ) -> object_rainbow::Result<()> {
        self.operator
            .write(key, hash.to_vec())
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))?;
        Ok(())
    }

    async fn fetch_ref(&self, key: &str) -> object_rainbow::Result<Hash> {
        match self.operator.read(key).await {
            Ok(value) => value
                .to_vec()
                .as_slice()
                .try_into()
                .map_err(|e| object_rainbow::error_parse!("{e}")),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Hash::default()),
            Err(e) => Err(object_rainbow::error_fetch!("{e}")),
        }
    }

    async fn ref_exists(&self, key: &str) -> object_rainbow::Result<bool> {
        self.operator
            .exists(key)
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))
    }
}
