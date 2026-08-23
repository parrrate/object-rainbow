use std::{path::Path, sync::Arc};

use async_fs::{File, OpenOptions};
use blocking::unblock;
use object_rainbow::fn_fetch::closure_fetch;

use crate::Chunks;

impl Chunks {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> object_rainbow::Result<Self> {
        let path = Arc::<Path>::from(path.as_ref());
        Self::from_seek(
            closure_fetch(path, async |path| Ok(File::open(path).await?)),
            unblock,
        )
        .await
    }

    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> object_rainbow::Result<()> {
        futures_util::io::copy(
            self.as_async_read(),
            &mut OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .await?,
        )
        .await?;
        Ok(())
    }
}
