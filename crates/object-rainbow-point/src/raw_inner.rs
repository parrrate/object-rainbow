use std::sync::Arc;

use object_rainbow::{
    Address, ByteNode, FailFuture, FetchBytes, Hash, InlineOutput, ListHashes, Output,
    ParseAsInline, ParseInline, PointInput, Resolve, Singular, Tagged, ToOutput,
    addressed::AddressedBytes,
};

use crate::{FromInner, RawPoint};

#[derive(Clone, ParseAsInline)]
pub struct RawPointInner {
    pub(crate) hash: Hash,
    pub(crate) fetch: Arc<dyn Send + Sync + FetchBytes>,
}

impl RawPointInner {
    pub fn cast<T, Extra: 'static + Clone>(self, extra: Extra) -> RawPoint<T, Extra> {
        RawPoint::from_inner(self, extra)
    }

    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self {
            hash: address.hash,
            fetch: Arc::new(AddressedBytes { address, resolve }),
        }
    }

    pub fn from_singular(singular: impl 'static + Singular) -> Self {
        Self {
            hash: singular.hash(),
            fetch: Arc::new(singular),
        }
    }
}

impl ToOutput for RawPointInner {
    fn to_output(&self, output: &mut impl Output) {
        self.hash.to_output(output);
    }
}

impl InlineOutput for RawPointInner {}

impl<I: PointInput> ParseInline<I> for RawPointInner {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(Self::from_address(input.parse_inline()?, input.resolve()))
    }
}

impl Tagged for RawPointInner {}

impl Singular for RawPointInner {
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl ListHashes for RawPointInner {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        f(self.hash)
    }

    fn point_count(&self) -> usize {
        1
    }
}

impl FetchBytes for RawPointInner {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.fetch.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.fetch.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.fetch.fetch_bytes_local()
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        self.fetch.fetch_data_local()
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.fetch.as_resolve()
    }

    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
        Arc::try_unwrap(self).ok()?.fetch.try_unwrap_resolve()
    }
}
