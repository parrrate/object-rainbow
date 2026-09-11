use std::sync::Arc;

use crate::*;

#[derive(Clone, Parse, ParseInline)]
pub struct AddressedBytes {
    pub address: Address,
    pub resolve: Arc<dyn Resolve>,
}

impl FetchBytes for AddressedBytes {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.resolve.resolve(self.address, &self.resolve)
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.resolve.resolve_data(self.address)
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.resolve.try_resolve_local(self.address, &self.resolve)
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        Some(&self.resolve)
    }

    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
        Arc::try_unwrap(self)
            .ok()
            .map(|Self { resolve, .. }| resolve)
    }
}

impl Singular for AddressedBytes {
    fn hash(&self) -> Hash {
        self.address.hash
    }
}

impl<I: PointInput> Parse<I> for Arc<dyn Singular> {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: PointInput> ParseInline<I> for Arc<dyn Singular> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Arc::new(input.parse_inline::<AddressedBytes>()?))
    }
}
