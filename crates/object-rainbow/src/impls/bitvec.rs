use bitvec::{array::BitArray, order::BitOrder, view::BitViewSized};

use crate::*;

#[derive(
    ToOutput, InlineOutput, Tagged, ListHashes, Topological, Parse, ParseInline, Size, MaybeHasNiche,
)]
#[rainbow(remote = "BitArray")]
struct __BitArray<A: BitViewSized, O: BitOrder> {
    pub _ord: PhantomData<O>,
    pub data: A,
}
