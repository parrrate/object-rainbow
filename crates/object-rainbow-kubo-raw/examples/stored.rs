use object_rainbow::Fetch;
use object_rainbow_kubo_raw::LocalIpfsStore;
use object_rainbow_point::IntoPoint;
use object_rainbow_store::RainbowStore;

#[tokio::main]
async fn main() -> object_rainbow::Result<()> {
    let store = LocalIpfsStore::default();
    let mut point = store
        .saved_point(
            &(*b"a", ((*b"b").point(), (*b"c").point()).point()).point(),
            (),
        )
        .await?;
    *point
        .fetch_mut()
        .await?
        .1
        .fetch_mut()
        .await?
        .1
        .fetch_mut()
        .await? = *b"d";
    point = store.saved_point(&point, ()).await?;
    assert_eq!(
        point.fetch().await?.1.fetch().await?.1.fetch().await?,
        *b"d",
    );
    Ok(())
}
