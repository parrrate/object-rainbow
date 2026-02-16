use std::collections::BTreeMap;

use bitvec::array::BitArray;
use object_rainbow::{
    Inline, InlineOutput, ListHashes, ParseAsInline, ParseInline, ParseInput, Tagged, ToOutput,
    Topological, assert_impl,
};

type Bits = BitArray<[u8; 32]>;

#[derive(ToOutput, Tagged, ListHashes, Topological, ParseAsInline)]
pub struct ArrayMap<T> {
    bits: Bits,
    map: BTreeMap<u8, T>,
}

impl<T: InlineOutput> InlineOutput for ArrayMap<T> {}

impl<T: ParseInline<I>, I: ParseInput> ParseInline<I> for ArrayMap<T> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let bits = input.parse_inline::<Bits>()?;
        let map = bits
            .iter_ones()
            .map(|one| {
                Ok((
                    u8::try_from(one).expect("overflow"),
                    input.parse_inline::<T>()?,
                ))
            })
            .collect::<object_rainbow::Result<_>>()?;
        Ok(Self { bits, map })
    }
}

assert_impl!(
    impl<T, E> Inline<E> for ArrayMap<T> where T: Inline<E> {}
);

impl<T> ArrayMap<T> {
    pub fn get_mut(&mut self, key: u8) -> Option<&mut T> {
        self.map.get_mut(&key)
    }
}
