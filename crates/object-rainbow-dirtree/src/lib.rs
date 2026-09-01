use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, ToOutput, assert_impl};
use object_rainbow_amt::AmtMap;

#[derive(Debug, Enum, ToOutput, InlineOutput)]
#[output(bound = "Segment: InlineOutput")]
pub enum DirEntry<Segment, File, Directory = ()> {
    File(File),
    Directory {
        #[output(unchecked)]
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
