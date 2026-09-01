use std::sync::Arc;

use object_rainbow::{Enum, InlineOutput, ListHashes, Tagged, ToOutput, assert_impl};
use object_rainbow_amt::AmtMap;

#[derive(Debug, Enum, ToOutput, InlineOutput, ListHashes, Tagged)]
#[output(bound = "Segment: InlineOutput")]
#[hashes(bound = "Segment: ListHashes")]
pub enum DirEntry<Segment, File, Directory = ()> {
    File(File),
    Directory {
        #[output(unchecked)]
        #[hashes(unchecked)]
        #[tags(replace = "Segment")]
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

assert_impl!(
    impl<Segment, File, Directory> ListHashes for DirEntry<Segment, File, Directory>
    where
        Segment: ListHashes,
        File: ListHashes,
        Directory: ListHashes,
    {
    }
);
