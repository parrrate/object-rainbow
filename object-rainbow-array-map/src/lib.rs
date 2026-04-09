use bitvec::array::BitArray;
use object_rainbow::{
    Inline, InlineOutput, ListHashes, ParseAsInline, ParseInline, ParseInput, Tagged, ToOutput,
    Topological, assert_impl,
};

type Bits = BitArray<[u8; 32]>;

#[derive(ToOutput, Tagged, ListHashes, Topological, ParseAsInline)]
pub struct ArrayMap<T> {
    map: Bits,
    values: Vec<T>,
}

impl<T: InlineOutput> InlineOutput for ArrayMap<T> {}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for ArrayMap<T> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let map = input.parse_inline::<Bits>()?;
        let values = input.parse_vec_n(map.count_ones())?;
        Ok(Self { map, values })
    }
}

assert_impl!(
    impl<T, E> Inline<E> for ArrayMap<T> where T: Inline<E> {}
);
