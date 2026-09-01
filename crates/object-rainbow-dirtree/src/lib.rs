use std::sync::Arc;

use object_rainbow::Enum;
use object_rainbow_amt::AmtMap;

#[derive(Debug, Enum)]
pub enum DirEntry<Segment, File, Directory = ()> {
    File(File),
    Directory {
        directory: Directory,
        children: AmtMap<Segment, Arc<Self>>,
    },
}
