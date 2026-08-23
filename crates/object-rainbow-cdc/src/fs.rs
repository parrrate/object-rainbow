use std::{path::Path, sync::Arc};

use async_fs::File;
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
}
