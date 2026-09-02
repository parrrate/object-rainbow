use std::path::Path;

use futures_util::TryStreamExt;
use object_rainbow::{Fetch, zero_terminated::Zt};
use object_rainbow_amt::AmtMap;
use object_rainbow_point::Point;

use crate::Chunks;

pub type FileMap = AmtMap<Zt<String>, Option<Point<Chunks>>>;

impl Chunks {
    pub async fn write_dir(dir: impl AsRef<Path>, map: FileMap) -> object_rainbow::Result<()> {
        let dir = dir.as_ref();
        if async_fs::metadata(dir).await.map(|_| false).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(true)
            } else {
                Err(e)
            }
        })? {
            async_fs::create_dir(dir).await?;
        }
        map.stream()
            .try_for_each_concurrent(None, async |(path, chunks)| {
                let path = dir.join(&*path);
                if let Some(chunks) = chunks {
                    chunks.fetch().await?.to_file(path).await?;
                } else {
                    async_fs::create_dir_all(path).await?;
                }
                Ok(())
            })
            .await?;
        Ok(())
    }
}
