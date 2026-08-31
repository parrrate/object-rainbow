use std::path::Path;

use async_walkdir::WalkDir;
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use object_rainbow::{Fetch, zero_terminated::Zt};
use object_rainbow_amt::AmtMap;
use object_rainbow_point::{IntoPoint, Point};

use crate::Chunks;

impl Chunks {
    pub async fn read_dir(
        dir: impl AsRef<Path>,
    ) -> object_rainbow::Result<AmtMap<Zt<String>, Option<Point<Self>>>> {
        let dir = dir.as_ref();
        let map = WalkDir::new(dir)
            .map_err(std::io::Error::from)
            .map_err(object_rainbow::Error::from)
            .try_filter_map(|entry| async move {
                let path = entry
                    .path()
                    .strip_prefix(dir)
                    .map_err(object_rainbow::Error::operation)?
                    .to_path_buf();
                let file_type = entry.file_type().await?;
                Ok(if file_type.is_file() {
                    Some((path, true))
                } else if file_type.is_dir() {
                    Some((path, false))
                } else {
                    None
                })
            })
            .map_ok(|(path, file)| {
                futures_util::stream::once(
                    async move {
                        Ok::<_, object_rainbow::Error>((
                            Zt::new(
                                path.to_str()
                                    .ok_or_else(|| object_rainbow::error_consistency!("not UTF-8"))?
                                    .to_owned(),
                            )?,
                            if file {
                                Some(Chunks::from_file(dir.join(&*path)).await?.point())
                            } else {
                                None
                            },
                        ))
                    }
                    .boxed(),
                )
            })
            .boxed()
            .try_flatten_unordered(None)
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .collect();
        Ok(map)
    }

    pub async fn write_dir(
        dir: impl AsRef<Path>,
        map: AmtMap<Zt<String>, Option<Point<Self>>>,
    ) -> object_rainbow::Result<()> {
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
