use std::path::Path;

use async_walkdir::WalkDir;
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use object_rainbow::zero_terminated::Zt;
use object_rainbow_amt::AmtMap;
use object_rainbow_point::{IntoPoint, Point};

use crate::Chunks;

impl Chunks {
    pub async fn read_dir(
        dir: impl AsRef<Path>,
    ) -> object_rainbow::Result<AmtMap<Zt<String>, Point<Self>>> {
        let dir = dir.as_ref();
        let map = WalkDir::new(dir)
            .map_err(std::io::Error::from)
            .map_err(object_rainbow::Error::from)
            .try_filter_map(|entry| async move {
                Ok(if entry.file_type().await?.is_file() {
                    Some(
                        entry
                            .path()
                            .strip_prefix(dir)
                            .map_err(object_rainbow::Error::operation)?
                            .to_path_buf(),
                    )
                } else {
                    None
                })
            })
            .map_ok(|path| {
                futures_util::stream::once(
                    async move {
                        let chunks = Chunks::from_file(dir.join(&*path)).await?;
                        Ok::<_, object_rainbow::Error>((
                            Zt::new(
                                path.to_str()
                                    .ok_or_else(|| object_rainbow::error_consistency!("not UTF-8"))?
                                    .to_owned(),
                            )?,
                            chunks.point(),
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
}
