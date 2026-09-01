use std::sync::Arc;

use object_rainbow_amt::AmtMap;

pub enum DirTree<Segment, File, Directory = ()> {
    File(File),
    Directory {
        directory: Directory,
        children: AmtMap<Segment, Arc<Self>>,
    },
}
