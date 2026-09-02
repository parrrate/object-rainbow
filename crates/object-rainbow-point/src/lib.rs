#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use futures_util::TryFutureExt;
pub use object_rainbow::extras::Extras;
use object_rainbow::{
    Address, ByteNode, CanonicalExtra, DefaultHash, Equivalent, ExtraFor, FailFuture, Fetch,
    FetchBytes, FullHash, Hash, InlineOutput, ListHashes, MaybeHasNiche, Node, OptionalHash,
    Output, Parse, ParseAsInline, ParseInline, PointInput, PointVisitor, Resolve, Singular,
    SingularFetch, Size, Tagged, ToOutput, Topological, Traversible,
    extras::fetch_extra::{ParseFetch, ParseFetchInline},
    object_marker::ObjectMarker,
};

#[cfg(feature = "serde")]
mod point_deserialize;
#[cfg(feature = "point-serialize")]
mod point_serialize;

#[derive(Clone)]
struct ByAddressInner {
    address: Address,
    resolve: Arc<dyn Resolve>,
}

impl FetchBytes for ByAddressInner {
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

impl Singular for ByAddressInner {
    fn hash(&self) -> Hash {
        self.address.hash
    }
}

struct ByAddress<T, Extra> {
    inner: ByAddressInner,
    extra: Extra,
    _object: PhantomData<fn() -> T>,
}

impl<T, Extra> ByAddress<T, Extra> {
    fn from_inner(inner: ByAddressInner, extra: Extra) -> Self {
        Self {
            inner,
            extra,
            _object: PhantomData,
        }
    }
}

impl<T, Extra> FetchBytes for ByAddress<T, Extra> {
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
        Arc::try_unwrap(self).ok().map(
            |Self {
                 inner: ByAddressInner { resolve, .. },
                 ..
             }| resolve,
        )
    }
}

impl<T, Extra: Send + Sync> Singular for ByAddress<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

impl<T: FullHash, Extra: Send + Sync + ExtraFor<T>> Fetch for ByAddress<T, Extra> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async {
            let (data, resolve) = self.fetch_bytes().await?;
            let object = self
                .extra
                .parse_checked(self.inner.address.hash, &data, &resolve)?;
            Ok((object, resolve))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (data, resolve) = self.fetch_bytes().await?;
            self.extra
                .parse_checked(self.inner.address.hash, &data, &resolve)
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((data, resolve)) = self.fetch_bytes_local()? else {
            return Ok(None);
        };
        let object = self
            .extra
            .parse_checked(self.inner.address.hash, &data, &resolve)?;
        Ok(Some((object, resolve)))
    }
}

struct FetchExtra<T, D> {
    inner: ByAddressInner,
    fetch: D,
    _object: PhantomData<fn() -> T>,
}

impl<T, D> FetchExtra<T, D> {
    fn from_inner(inner: ByAddressInner, fetch: D) -> Self {
        Self {
            inner,
            fetch,
            _object: PhantomData,
        }
    }
}

impl<T, D> FetchBytes for FetchExtra<T, D> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.inner.fetch_data()
    }
}

impl<T: FullHash, D: Fetch<T: Send + Sync + ExtraFor<T>>> FetchExtra<T, D> {
    async fn fetch_object(&self) -> object_rainbow::Result<Node<T>> {
        let ((data, resolve), extra) =
            futures_util::future::try_join(self.fetch_bytes(), self.fetch.fetch()).await?;
        let object = extra.parse_checked(self.inner.address.hash, &data, &resolve)?;
        Ok((object, resolve))
    }
}

impl<T: Send + FullHash, D: Fetch<T: Send + Sync + ExtraFor<T>>> Fetch for FetchExtra<T, D> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(self.fetch_object())
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (object, _) = self.fetch_object().await?;
            Ok(object)
        })
    }
}

trait FromInner {
    type Inner: 'static + Clone;
    type Extra: 'static + Clone;

    fn from_inner(inner: Self::Inner, extra: Self::Extra) -> Self;
}

trait InnerCast: FetchBytes {
    fn inner_cast<T: FromInner>(&self, extra: &T::Extra) -> Option<T> {
        self.as_inner()?
            .downcast_ref()
            .cloned()
            .map(|inner| T::from_inner(inner, extra.clone()))
    }
}

impl<T: ?Sized + FetchBytes> InnerCast for T {}

pub trait ExtractResolve: FetchBytes {
    fn extract_resolve<R: Any>(&self) -> Option<(&Address, &R)> {
        let ByAddressInner { address, resolve } =
            self.as_inner()?.downcast_ref::<ByAddressInner>()?;
        let resolve = resolve.as_ref().any_ref().downcast_ref::<R>()?;
        Some((address, resolve))
    }
}

impl<T: ?Sized + FetchBytes> ExtractResolve for T {}

#[derive(Clone, ParseAsInline)]
pub struct RawPointInner {
    hash: Hash,
    fetch: Arc<dyn Send + Sync + FetchBytes>,
}

impl RawPointInner {
    pub fn cast<T, Extra: 'static + Clone>(self, extra: Extra) -> RawPoint<T, Extra> {
        RawPoint::from_inner(self, extra)
    }

    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self {
            hash: address.hash,
            fetch: Arc::new(ByAddressInner { address, resolve }),
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

#[derive(ToOutput, InlineOutput, Tagged, Parse, ParseInline)]
pub struct RawPoint<T, Extra = ()> {
    inner: RawPointInner,
    extra: Extras<Extra>,
    object: ObjectMarker<T>,
}

impl<T, Extra: Clone> CanonicalExtra for RawPoint<T, Extra> {
    type Extra = Extra;

    fn canonical_extra(&self) -> Self::Extra {
        self.extra.canonical_extra()
    }
}

impl<T, Extra> ListHashes for RawPoint<T, Extra> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.inner.list_hashes(f);
    }

    fn topology_hash(&self) -> Hash {
        self.inner.topology_hash()
    }

    fn point_count(&self) -> usize {
        self.inner.point_count()
    }
}

impl<T, Extra: 'static + Clone> FromInner for RawPoint<T, Extra> {
    type Inner = RawPointInner;
    type Extra = Extra;

    fn from_inner(inner: Self::Inner, extra: Self::Extra) -> Self {
        RawPoint {
            inner,
            extra: Extras(extra),
            object: Default::default(),
        }
    }
}

impl<T, Extra: Clone> Clone for RawPoint<T, Extra> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            extra: self.extra.clone(),
            object: Default::default(),
        }
    }
}

impl<T: 'static + Traversible, Extra: 'static + Send + Sync + Clone + ExtraFor<T>> Topological
    for RawPoint<T, Extra>
{
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }
}

impl<T, Extra: Send + Sync> Singular for RawPoint<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

impl<T, Extra: 'static + Clone> RawPoint<T, Extra> {
    pub fn cast<U>(self) -> RawPoint<U, Extra> {
        self.inner.cast(self.extra.0)
    }
}

impl<T: 'static + FullHash, Extra: 'static + Send + Sync + ExtraFor<T>> RawPoint<T, Extra> {
    pub fn into_point(self) -> Point<T> {
        Point::from_singular(self)
    }
}

impl<T, Extra> FetchBytes for RawPoint<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.inner.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.inner.fetch_bytes_local()
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        self.inner.fetch_data_local()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.inner.as_resolve()
    }

    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
        Arc::try_unwrap(self).ok()?.inner.fetch.try_unwrap_resolve()
    }
}

impl<T: FullHash, Extra: Send + Sync + ExtraFor<T>> Fetch for RawPoint<T, Extra> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async {
            let (data, resolve) = self.inner.fetch.fetch_bytes().await?;
            let object = self
                .extra
                .0
                .parse_checked(self.inner.hash, &data, &resolve)?;
            Ok((object, resolve))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (data, resolve) = self.inner.fetch.fetch_bytes().await?;
            self.extra.0.parse_checked(self.inner.hash, &data, &resolve)
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((data, resolve)) = self.inner.fetch.fetch_bytes_local()? else {
            return Ok(None);
        };
        let object = self
            .extra
            .0
            .parse_checked(self.inner.hash, &data, &resolve)?;
        Ok(Some((object, resolve)))
    }
}

impl<T> Point<T> {
    pub async fn echo(fetch: impl 'static + Fetch<T = T>) -> object_rainbow::Result<Self>
    where
        T: FullHash,
    {
        Ok(Self::from_alternate_source(&fetch.fetch().await?, fetch))
    }

    pub fn from_alternate_source(object: &T, fetch: impl 'static + Fetch<T = T>) -> Self
    where
        T: FullHash,
    {
        Self::from_fetch(object.full_hash(), fetch)
    }

    fn from_trusted_fetch(hash: Hash, fetch: Arc<dyn Fetch<T = T>>) -> Self {
        Self {
            hash: hash.into(),
            fetch,
        }
    }

    pub fn from_fetch(hash: Hash, fetch: impl 'static + Fetch<T = T>) -> Self {
        Self::from_trusted_fetch(hash, fetch.into_dyn_fetch())
    }

    pub fn from_singular(singular: impl 'static + SingularFetch<T = T>) -> Self {
        Self::from_fetch(singular.hash(), singular)
    }

    fn map_fetch<U>(
        self,
        f: impl FnOnce(Arc<dyn Fetch<T = T>>) -> Arc<dyn Fetch<T = U>>,
    ) -> Point<U> {
        Point {
            hash: self.hash,
            fetch: f(self.fetch),
        }
    }
}

impl<U: 'static + Equivalent<T>, T: 'static, Extra> Equivalent<RawPoint<T, Extra>>
    for RawPoint<U, Extra>
{
    fn into_equivalent(self) -> RawPoint<T, Extra> {
        RawPoint {
            inner: self.inner,
            extra: self.extra,
            object: Default::default(),
        }
    }

    fn from_equivalent(object: RawPoint<T, Extra>) -> Self {
        Self {
            inner: object.inner,
            extra: object.extra,
            object: Default::default(),
        }
    }
}

#[derive(ParseAsInline, Tagged)]
#[must_use]
pub struct Point<T> {
    hash: OptionalHash,
    fetch: Arc<dyn Fetch<T = T>>,
}

impl<T> std::hash::Hash for Point<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T> std::fmt::Debug for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct Arc;
        f.debug_struct("Point")
            .field("hash", &self.hash)
            .field("fetch", &Arc)
            .finish()
    }
}

impl<T> Point<T> {
    pub fn raw<Extra: 'static + Clone>(self, extra: Extra) -> RawPoint<T, Extra> {
        {
            if let Some(raw) = self.fetch.inner_cast(&extra) {
                return raw;
            }
        }
        RawPointInner {
            hash: self.hash(),
            fetch: self.fetch,
        }
        .cast(extra)
    }
}

impl<T> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash().cmp(&other.hash())
    }
}

impl<T> Eq for Point<T> {}

impl<T> PartialEq for Point<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl<T> Clone for Point<T> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            fetch: self.fetch.clone(),
        }
    }
}

impl<T> Size for Point<T> {
    const SIZE: usize = Hash::SIZE;
    type Size = <Hash as Size>::Size;
}

impl<T: 'static + FullHash> Point<T>
where
    (): ExtraFor<T>,
{
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self::from_address_extra(address, resolve, ())
    }
}

impl<T: 'static + FullHash> Point<T> {
    pub fn from_address_extra<Extra: 'static + Send + Sync + Clone + ExtraFor<T>>(
        address: Address,
        resolve: Arc<dyn Resolve>,
        extra: Extra,
    ) -> Self {
        Self::from_trusted_fetch(
            address.hash,
            ByAddress::from_inner(ByAddressInner { address, resolve }, extra).into_dyn_fetch(),
        )
    }

    pub fn with_resolve<Extra: 'static + Send + Sync + Clone + ExtraFor<T>>(
        &self,
        resolve: Arc<dyn Resolve>,
        extra: Extra,
    ) -> Self {
        Self::from_address_extra(Address::from_hash(self.hash()), resolve, extra)
    }

    pub fn from_fetch_extra<Extra: 'static + Send + Sync + Clone + ExtraFor<T>>(
        address: Address,
        resolve: Arc<dyn Resolve>,
        fetch: impl 'static + Fetch<T = Extra>,
    ) -> Self
    where
        T: Send,
    {
        Self::from_trusted_fetch(
            address.hash,
            FetchExtra::from_inner(ByAddressInner { address, resolve }, fetch).into_dyn_fetch(),
        )
    }
}

impl<T> ListHashes for Point<T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        f(self.hash());
    }

    fn point_count(&self) -> usize {
        1
    }
}

impl<T: Traversible> Topological for Point<T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }
}

impl<T: 'static + FullHash, I: PointInput<Extra: Send + Sync + ExtraFor<T>>> ParseInline<I>
    for Point<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(Self::from_address_extra(
            input.parse_inline()?,
            input.resolve(),
            input.extra().clone(),
        ))
    }
}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.hash().to_output(output);
    }
}

impl<T> InlineOutput for Point<T> {}

impl<T> FetchBytes for Point<T> {
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

    fn as_inner(&self) -> Option<&dyn Any> {
        self.fetch.as_inner()
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.fetch.as_resolve()
    }

    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
        Arc::try_unwrap(self).ok()?.fetch.try_unwrap_resolve()
    }
}

impl<T> Singular for Point<T> {
    fn hash(&self) -> Hash {
        self.hash.unwrap()
    }
}

impl<T> Point<T> {
    pub fn get(&self) -> Option<&T> {
        self.fetch.get()
    }

    pub fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<T>>> {
        self.fetch.try_fetch_local()
    }

    pub fn try_unwrap(self) -> Option<T> {
        self.fetch.try_unwrap()
    }

    pub fn fetch(&self) -> FailFuture<'_, T> {
        self.fetch.fetch()
    }
}

impl<T: Traversible + Clone> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self::from_trusted_fetch(object.full_hash(), object.local_fetch())
    }

    fn yolo_mut(&mut self) -> bool {
        self.fetch.get().is_some()
            && Arc::get_mut(&mut self.fetch).is_some_and(|fetch| fetch.get_mut().is_some())
    }

    async fn prepare_yolo_fetch(&mut self) -> object_rainbow::Result<()> {
        if !self.yolo_mut() {
            let object = self.fetch.fetch().await?;
            self.fetch = object.local_fetch();
        }
        Ok(())
    }

    pub async fn fetch_mut(&'_ mut self) -> object_rainbow::Result<PointMut<'_, T>> {
        self.prepare_yolo_fetch().await?;
        let fetch = Arc::get_mut(&mut self.fetch).expect("shared fetch?");
        assert!(fetch.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            fetch,
        })
    }

    pub async fn fetch_ref(&mut self) -> object_rainbow::Result<&T> {
        self.prepare_yolo_fetch().await?;
        Ok(self.fetch.get().expect("non-local fetch"))
    }

    pub async fn fetch_take(&mut self) -> object_rainbow::Result<T>
    where
        T: Default,
    {
        Ok(std::mem::take(&mut *self.fetch_mut().await?))
    }
}

impl<T: FullHash> Fetch for Point<T> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        self.fetch.fetch_full()
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        self.fetch.fetch()
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        self.fetch.try_fetch_local()
    }

    fn fetch_local(&self) -> Option<Self::T> {
        self.fetch.fetch_local()
    }

    fn get(&self) -> Option<&Self::T> {
        self.fetch.get()
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        self.hash.clear();
        Arc::get_mut(&mut self.fetch)?.get_mut()
    }

    fn get_mut_finalize(&mut self) {
        let fetch = Arc::get_mut(&mut self.fetch).expect("shared fetch?");
        fetch.get_mut_finalize();
        self.hash = fetch.get().expect("non-local fetch").full_hash().into();
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok()?.fetch.try_unwrap()
    }

    fn into_dyn_fetch<'a>(self) -> Arc<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a + Sized,
    {
        self.fetch
    }
}

/// This implementation is the main goal of [`Equivalent`]: we assume transmuting the pointer is
/// safe.
impl<U: 'static + Equivalent<T>, T: 'static> Equivalent<Point<T>> for Point<U> {
    fn into_equivalent(self) -> Point<T> {
        self.map_fetch(|fetch| {
            MapEquivalent {
                fetch,
                map: U::into_equivalent,
            }
            .into_dyn_fetch()
        })
    }

    fn from_equivalent(point: Point<T>) -> Self {
        point.map_fetch(|fetch| {
            MapEquivalent {
                fetch,
                map: U::from_equivalent,
            }
            .into_dyn_fetch()
        })
    }
}

impl<T> MaybeHasNiche for Point<T> {
    type MnArray = <Hash as MaybeHasNiche>::MnArray;
}

impl<T: DefaultHash> Point<T> {
    pub fn is_default(&self) -> bool {
        self.hash() == T::default_hash()
    }
}

impl<T: Default + Traversible + Clone> Default for Point<T> {
    fn default() -> Self {
        T::default().point()
    }
}

pub trait IntoPoint: Traversible {
    fn point(self) -> Point<Self>
    where
        Self: Clone,
    {
        Point::from_object(self)
    }
}

impl<T: Traversible> IntoPoint for T {}

struct MapEquivalent<T, F> {
    fetch: Arc<dyn Fetch<T = T>>,
    map: F,
}

impl<T, F> FetchBytes for MapEquivalent<T, F> {
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

trait Map1<T>: Fn(T) -> Self::U {
    type U;
}

impl<T, U, F: Fn(T) -> U> Map1<T> for F {
    type U = U;
}

impl<T, F: Send + Sync + Map1<T>> Fetch for MapEquivalent<T, F> {
    type T = F::U;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(self.fetch.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(self.fetch.fetch().map_ok(&self.map))
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((object, resolve)) = self.fetch.try_fetch_local()? else {
            return Ok(None);
        };
        let object = (self.map)(object);
        Ok(Some((object, resolve)))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        self.fetch.fetch_local().map(&self.map)
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        let Self { fetch, map } = Arc::try_unwrap(self).ok()?;
        fetch.try_unwrap().map(map)
    }
}

pub struct PointMut<'a, T: FullHash> {
    hash: &'a mut OptionalHash,
    fetch: &'a mut dyn Fetch<T = T>,
}

impl<T: FullHash> Deref for PointMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.fetch.get().expect("non-local fetch")
    }
}

impl<T: FullHash> DerefMut for PointMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fetch.get_mut().expect("non-local fetch")
    }
}

impl<T: FullHash> Drop for PointMut<'_, T> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.finalize();
        }
    }
}

impl<'a, T: FullHash> PointMut<'a, T> {
    fn finalize(&mut self) {
        self.fetch.get_mut_finalize();
        *self.hash = self.full_hash().into();
    }
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, Tagged, Parse, ParseInline)]
pub struct ExtraPoint<T, Extra = ()> {
    pub extra: Extras<Extra>,
    pub point: Point<T>,
}

impl<T, Extra: std::fmt::Debug> std::fmt::Debug for ExtraPoint<T, Extra> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtraPoint")
            .field("extra", &self.extra)
            .field("point", &self.point)
            .finish()
    }
}

impl<T, Extra: Clone> Clone for ExtraPoint<T, Extra> {
    fn clone(&self) -> Self {
        Self {
            extra: self.extra.clone(),
            point: self.point.clone(),
        }
    }
}

impl<T, Extra: PartialEq> PartialEq for ExtraPoint<T, Extra> {
    fn eq(&self, other: &Self) -> bool {
        self.extra == other.extra && self.point == other.point
    }
}

impl<T, Extra: Clone> CanonicalExtra for ExtraPoint<T, Extra> {
    type Extra = Extra;

    fn canonical_extra(&self) -> Self::Extra {
        self.extra.canonical_extra()
    }
}

impl<T, E> FetchBytes for ExtraPoint<T, E> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.point.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.point.fetch_data()
    }
}

impl<T: FullHash, E: Send + Sync> Fetch for ExtraPoint<T, E> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        self.point.fetch_full()
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        self.point.fetch()
    }
}

impl<T: 'static + Send + FullHash, E: 'static + Send + Sync + Clone + ExtraFor<T>> ParseFetch<E>
    for Point<T>
{
    fn parse_fetch<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> object_rainbow::Result<Self> {
        Self::parse_fetch_as_inline(input)
    }
}

impl<T: 'static + Send + FullHash, E: 'static + Send + Sync + Clone + ExtraFor<T>>
    ParseFetchInline<E> for Point<T>
{
    fn parse_fetch_inline<I: PointInput<Extra: Fetch<T = E>>>(
        input: &mut I,
    ) -> object_rainbow::Result<Self> {
        Ok(Self::from_fetch_extra(
            input.parse_inline()?,
            input.resolve(),
            input.extra().clone(),
        ))
    }
}
