use std::convert::Infallible;

use object_rainbow::Fetch;
use object_rainbow_encrypted::{Key, encrypt_point};
use object_rainbow_kubo_raw::LocalIpfsStore;
use object_rainbow_point::IntoPoint;
use object_rainbow_store::RainbowStore;

#[derive(Clone, Copy, PartialEq, Eq)]
struct InverseKey;

impl Key for InverseKey {
    type Error = Infallible;

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        println!("encrypting");
        data.iter().copied().map(|x| !x).collect()
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(data.iter().copied().map(|x| !x).collect())
    }
}

#[tokio::main]
async fn main() -> object_rainbow::Result<()> {
    let store = LocalIpfsStore::default();
    let mut point = encrypt_point(
        InverseKey,
        (*b"a", ((*b"b").point(), (*b"c").point()).point()).point(),
    )
    .await?;
    println!("encrypted");
    point = store.saved_point(&point, InverseKey).await?;
    println!("saved");
    {
        let mut enc = point.fetch_mut().await?;
        let mut enc = enc.as_mut();
        *enc.1.fetch_mut().await?.1.fetch_mut().await? = *b"d";
        enc.save().await?;
    }
    println!("encrypted 2");
    point = store.saved_point(&point, InverseKey).await?;
    println!("saved 2");
    assert_eq!(
        point.fetch().await?.1.fetch().await?.1.fetch().await?,
        *b"d",
    );
    Ok(())
}
