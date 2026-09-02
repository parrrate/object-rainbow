use std::{path::Path, sync::Arc};

use futures_util::{StreamExt, TryStreamExt};
use object_rainbow::zero_terminated::Zt;
use object_rainbow_dirtree::DirEntry;
use object_rainbow_point::{IntoPoint, Point};

use crate::Chunks;

pub type FileTree = DirEntry<Zt<String>, Point<Chunks>>;

impl Chunks {
    fn read_tree_inner(
        path: impl Send + AsRef<Path>,
    ) -> impl Send + Future<Output = object_rainbow::Result<FileTree>> {
        Self::read_tree(path)
    }

    pub async fn read_tree(path: impl AsRef<Path>) -> object_rainbow::Result<FileTree> {
        let path = path.as_ref().to_path_buf();
        let path = &*path;
        let file_type = async_fs::metadata(path).await?.file_type();
        if file_type.is_file() {
            let chunks = Chunks::from_file(path).await?.point();
            Ok(DirEntry::File(chunks))
        } else if file_type.is_dir() {
            let children = async_fs::read_dir(path)
                .await?
                .map_ok(|entry| {
                    futures_util::stream::once(async move {
                        let p = entry.path();
                        let segment = Zt::new(
                            p.strip_prefix(path)
                                .map_err(object_rainbow::Error::operation)?
                                .as_os_str()
                                .to_str()
                                .ok_or_else(|| object_rainbow::error_consistency!("not UTF-8"))?
                                .to_owned(),
                        )?;
                        let tree = Arc::new(Chunks::read_tree_inner(p).await?);
                        Ok::<_, object_rainbow::Error>((segment, tree))
                    })
                    .boxed()
                })
                .boxed()
                .try_flatten_unordered(None)
                .try_collect::<Vec<_>>()
                .await?
                .into_iter()
                .collect();
            Ok(DirEntry::Directory {
                children,
                directory: (),
            })
        } else {
            Err(object_rainbow::Error::Unimplemented)
        }
    }
}
