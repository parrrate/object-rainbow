use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, ToOutput, assert_impl};
use object_rainbow_amt::AmtMap;

#[derive(Debug, Enum, ToOutput)]
#[output(unchecked)]
#[output(bound = "Segment: InlineOutput")]
#[output(bound = "File: ToOutput")]
#[output(bound = "Directory: ToOutput")]
pub enum DirEntry<Segment, File, Directory = ()> {
    File(File),
    Directory {
        children: AmtMap<Segment, Arc<Self>>,
        directory: Directory,
    },
}

assert_impl!(
    impl<Segment, File, Directory> ToOutput for DirEntry<Segment, File, Directory>
    where
        Segment: InlineOutput,
        File: InlineOutput,
        Directory: InlineOutput,
    {
    }
);
