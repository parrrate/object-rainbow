use std::sync::Arc;

use bytes::Bytes;
use cid::{Cid, multihash::Multihash};
use object_rainbow::{Hash, ToOutput, WithHash};
use object_rainbow_store::RainbowStore;
use reqwest::{
    Client,
    multipart::{Form, Part},
};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use url::Url;

#[derive(Clone)]
pub struct LocalIpfsStore {
    client: Client,
    url: Arc<Url>,
}

impl PartialEq for LocalIpfsStore {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.url, &other.url)
    }
}

impl Default for LocalIpfsStore {
    fn default() -> Self {
        Self {
            client: Default::default(),
            url: Arc::new("http://127.0.0.1:5001".parse().unwrap()),
        }
    }
}

#[serde_as]
#[derive(Deserialize)]
struct RootCid {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "/")]
    root: Cid,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PutResponse {
    cid: RootCid,
}

impl LocalIpfsStore {
    async fn gat_put(&self, data: Vec<u8>) -> object_rainbow::Result<Cid> {
        let hash = data.data_hash();
        let mut url = self.url.as_ref().clone();
        url.set_path("/api/v0/dag/put");
        url.query_pairs_mut()
            .append_pair("store-codec", "raw")
            .append_pair("input-codec", "raw")
            .append_pair("pin", "true")
            .append_pair("hash", "sha2-256");
        let response = self
            .client
            .post(url)
            .multipart(Form::new().part("file", Part::bytes(data)))
            .send()
            .await
            .map_err(object_rainbow::Error::fetch)?;
        let status = response.status();
        if status.is_success() {
            let PutResponse { cid } = response
                .json()
                .await
                .map_err(object_rainbow::Error::fetch)?;
            let cid = cid.root;
            if cid.hash().digest() == *hash {
                Ok(cid)
            } else {
                Err(object_rainbow::error_consistency!(
                    "CID doesn't match hash of the data",
                ))
            }
        } else {
            let mut text = response
                .text()
                .await
                .map_err(object_rainbow::Error::fetch)?;
            if text.trim().is_empty() {
                text.push_str("(empty or whitespace-only body)");
            }
            Err(object_rainbow::error_fetch!("{status}: {text}"))
        }
    }

    async fn dag_get(&self, cid: Cid) -> object_rainbow::Result<Bytes> {
        let mut url = self.url.as_ref().clone();
        url.set_path("/api/v0/dag/get");
        url.query_pairs_mut()
            .append_pair("arg", &cid.to_string())
            .append_pair("output-codec", "raw");
        let response = self
            .client
            .post(url)
            .send()
            .await
            .map_err(object_rainbow::Error::fetch)?;
        let status = response.status();
        if status.is_success() {
            Ok(response
                .bytes()
                .await
                .map_err(object_rainbow::Error::fetch)?)
        } else {
            let mut text = response
                .text()
                .await
                .map_err(object_rainbow::Error::fetch)?;
            if text.trim().is_empty() {
                text.push_str("(empty or whitespace-only body)");
            }
            Err(object_rainbow::error_fetch!("{status}: {text}"))
        }
    }
}

impl RainbowStore for LocalIpfsStore {
    async fn save_data(
        &self,
        wh: WithHash<'_, impl Send + Sync + ToOutput>,
    ) -> object_rainbow::Result<()> {
        self.gat_put(wh.vec()).await?;
        Ok(())
    }

    /// IPFS seems to be blocking indefinitely when something is not found, which is suboptimal for
    /// an operation that's supposed to be an optimisation.
    async fn contains(&self, _: Hash) -> object_rainbow::Result<bool> {
        Ok(false)
    }

    #[expect(refining_impl_trait)]
    async fn fetch(&self, hash: Hash) -> object_rainbow::Result<Bytes> {
        let mut multihash = [0; 34];
        multihash[0] = 0x12;
        multihash[1] = 0x20;
        multihash[2..].copy_from_slice(&*hash);
        Ok(self
            .dag_get(Cid::new_v1(
                0x55,
                Multihash::from_bytes(&multihash).map_err(object_rainbow::Error::parse)?,
            ))
            .await?
            .slice(32..))
    }
}
