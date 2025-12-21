use object_rainbow::ToOutput;
use object_rainbow_store::RainbowStore;
use opendal::Operator;

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
    async fn save_data(
        &self,
        hashes: object_rainbow::ObjectHashes,
        data: &[u8],
    ) -> object_rainbow::Result<()> {
        self.operator
            .write(&hex::encode(hashes.data_hash()), data.to_vec())
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))?;
        Ok(())
    }

    async fn contains(&self, hash: object_rainbow::Hash) -> object_rainbow::Result<bool> {
        self.operator
            .exists(&hex::encode(hash))
            .await
            .map_err(|e| object_rainbow::error_fetch!("{e}"))
    }

    async fn fetch(
        &self,
        hash: object_rainbow::Hash,
    ) -> object_rainbow::Result<impl Send + Sync + AsRef<[u8]>> {
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
