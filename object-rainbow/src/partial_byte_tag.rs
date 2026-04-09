use std::marker::PhantomData;

use typenum::{ToInt, U1};

use crate::{enumkind::UsizeTag, incr_byte_niche::IncrByteNiche, *};

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Topological, ParseAsInline)]
pub struct PartialByteTag<K>(u8, PhantomData<K>);

impl<K> Size for PartialByteTag<K> {
    type Size = U1;
    const SIZE: usize = 1;
}

impl<K: ToInt<u8>, E: ParseInput> ParseInline<E> for PartialByteTag<K> {
    fn parse_inline(input: &mut E) -> crate::Result<Self> {
        let n = input.parse_inline()?;
        if n < K::INT {
            Ok(Self(n, PhantomData))
        } else {
            Err(crate::Error::OutOfBounds)
        }
    }
}

impl<K: ToInt<u8>> UsizeTag for PartialByteTag<K> {
    fn from_usize(n: usize) -> Self {
        assert!(n < K::INT as usize);
        Self(n as _, PhantomData)
    }

    fn to_usize(&self) -> usize {
        self.0.to_usize()
    }

    fn try_to_usize(&self) -> Option<usize> {
        self.0.try_to_usize()
    }
}

impl<K> MaybeHasNiche for PartialByteTag<K> {
    type MnArray = SomeNiche<IncrByteNiche<K>>;
}
