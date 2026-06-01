use std::sync::Arc;

use object_rainbow::{Hash, OptionalHash, ParseSliceRefless, ToOutput, WithHash};
use object_rainbow_store::{RainbowStore, RainbowStoreMut};
use opendal::{ErrorKind, Operator};

#[derive(Debug, Clone)]
pub struct OpendalStore {
    operator: Operator,
    ptr: Arc<()>,
}

impl OpendalStore {
    pub fn from_operator(operator: Operator) -> Self {
        Self {
            operator,
            ptr: Default::default(),
        }
    }
}

impl PartialEq for OpendalStore {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.ptr, &other.ptr)
    }
}

fn to_key(hash: Hash) -> String {
    hex::encode(hash)
}

impl RainbowStore for OpendalStore {
    async fn save_data(
        &self,
        wh: WithHash<'_, impl Send + Sync + ToOutput>,
    ) -> object_rainbow::Result<()> {
        self.operator
            .write(&to_key(wh.data_hash()), wh.data.vec())
            .await
            .map_err(object_rainbow::Error::io)?;
        Ok(())
    }

    async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        self.operator
            .exists(&to_key(hash))
            .await
            .map_err(object_rainbow::Error::io)
    }

    async fn fetch(
        &self,
        hash: Hash,
    ) -> object_rainbow::Result<impl 'static + Send + Sync + AsRef<[u8]>> {
        self.operator
            .read(&to_key(hash))
            .await
            .map_err(object_rainbow::Error::io)
            .map(|b| b.to_bytes())
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
            .map_err(object_rainbow::Error::io)?;
        Ok(())
    }

    async fn fetch_ref(&self, key: &str) -> object_rainbow::Result<OptionalHash> {
        match self.operator.read(key).await {
            Ok(value) => OptionalHash::parse_slice_refless(&value.to_vec()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Default::default()),
            Err(e) => Err(object_rainbow::Error::io(e)),
        }
    }

    async fn ref_exists(&self, key: &str) -> object_rainbow::Result<bool> {
        self.operator
            .exists(key)
            .await
            .map_err(object_rainbow::Error::io)
    }
}
