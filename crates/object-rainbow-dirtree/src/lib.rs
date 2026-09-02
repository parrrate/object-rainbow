use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Tagged, ToOutput, Topological, Traversible, assert_impl,
};
use object_rainbow_amt::AmtMap;

#[derive(Debug, Enum, ToOutput, InlineOutput, ListHashes, Tagged, Topological)]
#[output(bound = "Segment: InlineOutput")]
#[hashes(bound = "Segment: ListHashes")]
#[topology(unchecked)]
#[topology(bound = "Segment: InlineOutput + Traversible")]
#[topology(bound = "File: InlineOutput + Traversible")]
#[topology(bound = "Directory: InlineOutput + Traversible")]
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
    impl<Segment, File, Directory> InlineOutput for DirEntry<Segment, File, Directory>
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

assert_impl!(
    impl<Segment, File, Directory> Tagged for DirEntry<Segment, File, Directory>
    where
        Segment: Tagged,
        File: Tagged,
        Directory: Tagged,
    {
    }
);
