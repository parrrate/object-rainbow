use object_rainbow::zero_terminated::Zt;
use object_rainbow_dirtree::DirEntry;

use crate::Chunks;

pub type FileTree = DirEntry<Zt<String>, Chunks>;
