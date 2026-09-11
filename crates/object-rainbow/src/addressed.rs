use std::sync::Arc;

use crate::{extras::Extras, object_marker::ObjectMarker, *};

pub trait ExtractResolve: FetchBytes {
    fn extract_resolve<R: Any>(&self) -> Option<(&Address, &R)> {
        let AddressedBytes { address, resolve } =
            self.as_inner()?.downcast_ref::<AddressedBytes>()?;
        let resolve = resolve.as_ref().any_ref().downcast_ref::<R>()?;
        Some((address, resolve))
    }
}

impl<T: ?Sized + FetchBytes> ExtractResolve for T {}

#[derive(
    Clone, ToOutput, InlineOutput, Tagged, ListHashes, Parse, ParseInline, Size, MaybeHasNiche,
)]
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

#[derive(ToOutput, InlineOutput, Tagged, ListHashes, Parse, ParseInline, Size, MaybeHasNiche)]
pub struct Addressed<T, Extra> {
    inner: AddressedBytes,
    extra: Extras<Extra>,
    _object: ObjectMarker<T>,
}

impl<T: Traversible, Extra: 'static + Send + Sync + Clone + ExtraFor<T>> Topological
    for Addressed<T, Extra>
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }
}

impl<T, Extra: Clone> Clone for Addressed<T, Extra> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            extra: self.extra.clone(),
            _object: Default::default(),
        }
    }
}

impl<T, Extra> Addressed<T, Extra> {
    pub fn from_inner(inner: AddressedBytes, extra: Extra) -> Self {
        Self {
            inner,
            extra: Extras(extra),
            _object: Default::default(),
        }
    }
}

impl<T, Extra> FetchBytes for Addressed<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.inner.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.inner.fetch_bytes_local()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.inner.as_resolve()
    }

    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
        Arc::try_unwrap(self)
            .ok()
            .map(|Self { inner, .. }| inner.resolve)
    }
}

impl<T, Extra: Send + Sync> Singular for Addressed<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

impl<T: FullHash, Extra: Send + Sync + ExtraFor<T>> Fetch for Addressed<T, Extra> {
    type T = T;

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (data, resolve) = self.fetch_bytes().await?;
            self.extra.parse_checked(self.inner.hash(), &data, &resolve)
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((data, resolve)) = self.fetch_bytes_local()? else {
            return Ok(None);
        };
        let object = self
            .extra
            .parse_checked(self.inner.hash(), &data, &resolve)?;
        Ok(Some((object, resolve)))
    }
}
