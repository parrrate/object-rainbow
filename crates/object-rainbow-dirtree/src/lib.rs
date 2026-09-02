use std::sync::Arc;

use object_rainbow::{
    Enum, Inline, InlineOutput, ListHashes, Parse, ParseInline, PointInput, Tagged, ToOutput,
    Topological, Traversible, assert_impl,
};
use object_rainbow_amt::AmtMap;

#[derive(
    Debug, Enum, ToOutput, InlineOutput, ListHashes, Tagged, Topological, Parse, ParseInline,
)]
#[output(bound = "Segment: InlineOutput")]
#[hashes(bound = "Segment: ListHashes")]
#[topology(unchecked)]
#[topology(bound = "Segment: InlineOutput + Traversible")]
#[topology(bound = "File: InlineOutput + Traversible")]
#[topology(bound = "Directory: InlineOutput + Traversible")]
#[parse(input = "I", unchecked)]
#[parse(generic = "E: 'static + Send + Sync + Clone")]
#[parse(bound = "I: PointInput<Extra = E>")]
#[parse(bound = "Segment: ParseInline<I> + Inline<E>")]
#[parse(bound = "File: ParseInline<I> + Inline<E>")]
#[parse(bound = "Directory: ParseInline<I> + Inline<E>")]
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
