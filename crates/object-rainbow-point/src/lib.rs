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
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::pin(async move { self.resolve.resolve(self.address, &self.resolve).await })
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::pin(async move { self.resolve.resolve_data(self.address).await })
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        Some(&self.resolve)
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

impl<T, Extra: Clone> Clone for ByAddress<T, Extra> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            extra: self.extra.clone(),
            _object: PhantomData,
        }
    }
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
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_data()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.inner.as_resolve()
    }

    fn try_unwrap_resolve(self: Box<Self>) -> Option<Arc<dyn Resolve>> {
        Some(self.inner.resolve)
    }
}

impl<T, Extra: Send + Sync> Singular for ByAddress<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

impl<T: FullHash, Extra: Send + Sync + Clone + ExtraFor<T>> Fetch for ByAddress<T, Extra> {
    type T = T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let hash = self.inner.address.hash;
            let extra = self.extra.clone();
            let (data, resolve) = self.fetch_bytes().await?;
            extra.parse_checked(hash, &data, &resolve)
        })
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
    }
}

struct FetchExtra<T, D> {
    inner: ByAddressInner,
    fetch: Box<D>,
    _object: PhantomData<fn() -> T>,
}

impl<T, D: Clone> Clone for FetchExtra<T, D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            fetch: self.fetch.clone(),
            _object: PhantomData,
        }
    }
}

impl<T, D> FetchExtra<T, D> {
    fn from_inner(inner: ByAddressInner, fetch: D) -> Self {
        let fetch = fetch.into();
        Self {
            inner,
            fetch,
            _object: PhantomData,
        }
    }
}

impl<T, D> FetchBytes for FetchExtra<T, D> {
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_data()
    }
}

impl<T: FullHash, D: Clone + Fetch<T: Send + Sync + ExtraFor<T>>> FetchExtra<T, D> {
    async fn fetch_object(self) -> object_rainbow::Result<Node<T>> {
        let hash = self.inner.address.hash;
        let ((data, resolve), extra) = futures_util::future::try_join(
            Box::new(self.clone()).fetch_bytes(),
            self.fetch.clone().fetch(),
        )
        .await?;
        let object = extra.parse_checked(hash, &data, &resolve)?;
        Ok((object, resolve))
    }
}

impl<T: Send + FullHash, D: Clone + Fetch<T: Send + Sync + ExtraFor<T>>> Fetch
    for FetchExtra<T, D>
{
    type T = T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        Box::pin(async {
            let (object, _) = self.fetch_object().await?;
            Ok(object)
        })
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
    }
}

trait FromInner {
    type Inner: 'static;
    type Extra: 'static + Clone;

    fn from_inner(inner: Self::Inner, extra: Self::Extra) -> Self;
}

pub trait ExtractResolve: FetchBytes {
    fn extract_resolve<R: Any>(&self) -> Option<(&Address, &R)> {
        let ByAddressInner { address, resolve } =
            self.as_inner()?.downcast_ref::<ByAddressInner>()?;
        let resolve = resolve.as_ref().any_ref().downcast_ref::<R>()?;
        Some((address, resolve))
    }
}

impl<T: ?Sized + FetchBytes> ExtractResolve for T {}

#[derive(ParseAsInline)]
pub struct RawPointInner {
    hash: Hash,
    fetch: Box<dyn Send + Sync + FetchBytes>,
}

impl RawPointInner {
    pub fn cast<T, Extra: 'static + Clone>(self, extra: Extra) -> RawPoint<T, Extra> {
        RawPoint::from_inner(self, extra)
    }

    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self {
            hash: address.hash,
            fetch: Box::new(ByAddressInner { address, resolve }),
        }
    }

    pub fn from_singular(singular: impl 'static + Singular) -> Self {
        Self {
            hash: singular.hash(),
            fetch: Box::new(singular),
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
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::new(self.fetch).fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::new(self.fetch).fetch_data()
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.fetch.as_resolve()
    }

    fn try_unwrap_resolve(self: Box<Self>) -> Option<Arc<dyn Resolve>> {
        self.fetch.try_unwrap_resolve()
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

impl<T, Extra> FetchBytes for RawPoint<T, Extra> {
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::new(self.inner).fetch_data()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.inner.as_resolve()
    }

    fn try_unwrap_resolve(self: Box<Self>) -> Option<Arc<dyn Resolve>> {
        self.inner.fetch.try_unwrap_resolve()
    }
}

impl<T> Point<T> {
    pub fn from_alternate_source(object: &T, fetch: impl 'static + Fetch<T = T>) -> Self
    where
        T: FullHash,
    {
        Self::from_fetch(object.full_hash(), fetch)
    }

    fn from_trusted_fetch(hash: Hash, fetch: Box<dyn Fetch<T = T>>) -> Self {
        Self {
            hash: hash.into(),
            fetch: Some(fetch),
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
        f: impl FnOnce(Box<dyn Fetch<T = T>>) -> Box<dyn Fetch<T = U>>,
    ) -> Point<U> {
        Point {
            hash: self.hash,
            fetch: Some(f(self.fetch.unwrap())),
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
    fetch: Option<Box<dyn Fetch<T = T>>>,
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

impl<T: 'static + Clone> Clone for Point<T> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            fetch: Some(self.fetch.as_ref().unwrap().clone_boxed()),
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
        fetch: impl 'static + Clone + Fetch<T = Extra>,
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
    fn traverse(self, visitor: &mut impl PointVisitor) {
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
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        self.fetch.unwrap().fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        self.fetch.unwrap().fetch_data()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        self.fetch.as_ref().unwrap().as_inner()
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.fetch.as_ref().unwrap().as_resolve()
    }

    fn try_unwrap_resolve(self: Box<Self>) -> Option<Arc<dyn Resolve>> {
        self.fetch.unwrap().try_unwrap_resolve()
    }
}

impl<T> Singular for Point<T> {
    fn hash(&self) -> Hash {
        self.hash.unwrap()
    }
}

impl<T> Point<T> {
    pub fn get(&self) -> Option<&T> {
        self.fetch.as_ref().unwrap().get()
    }

    pub fn fetch<'a>(self) -> FailFuture<'a, T>
    where
        Self: 'a,
    {
        self.fetch.unwrap().fetch()
    }
}

impl<T: Traversible> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self::from_trusted_fetch(object.full_hash(), object.local_fetch())
    }

    fn yolo_mut(&mut self) -> bool {
        self.fetch.as_ref().unwrap().get().is_some()
            && self.fetch.as_mut().unwrap().get_mut().is_some()
    }

    async fn prepare_yolo_fetch(&mut self) -> object_rainbow::Result<()> {
        if !self.yolo_mut() {
            let object = self.fetch.take().unwrap().fetch().await?;
            self.fetch = Some(object.local_fetch());
        }
        Ok(())
    }

    pub async fn fetch_mut(&'_ mut self) -> object_rainbow::Result<PointMut<'_, T>> {
        self.prepare_yolo_fetch().await?;
        let fetch = &mut **self.fetch.as_mut().unwrap();
        assert!(fetch.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            fetch,
        })
    }

    pub async fn fetch_ref(&mut self) -> object_rainbow::Result<&T> {
        self.prepare_yolo_fetch().await?;
        Ok(self.fetch.as_ref().unwrap().get().expect("non-local fetch"))
    }

    pub async fn fetch_take(&mut self) -> object_rainbow::Result<T>
    where
        T: Default,
    {
        Ok(std::mem::take(&mut *self.fetch_mut().await?))
    }
}

impl<T: 'static + FullHash> Fetch for Point<T> {
    type T = T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        self.fetch.unwrap().fetch()
    }

    fn get(&self) -> Option<&Self::T> {
        self.fetch.as_ref().unwrap().get()
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        let object = self.fetch.as_mut().unwrap().get_mut()?;
        self.hash.clear();
        Some(object)
    }

    fn get_mut_finalize(&mut self) {
        let fetch = self.fetch.as_mut().unwrap();
        fetch.get_mut_finalize();
        self.hash = fetch.get().expect("non-local fetch").full_hash().into();
    }

    fn try_unwrap(self: Box<Self>) -> Option<Self::T> {
        self.fetch.unwrap().try_unwrap()
    }

    fn into_dyn_fetch<'a>(self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a + Sized,
    {
        self.fetch.unwrap()
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
    }
}

/// This implementation is the main goal of [`Equivalent`]: we assume transmuting the pointer is
/// safe.
impl<U: 'static + Clone + Equivalent<T>, T: 'static + Clone> Equivalent<Point<T>> for Point<U> {
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
    fn point(self) -> Point<Self> {
        Point::from_object(self)
    }
}

impl<T: Traversible> IntoPoint for T {}

struct MapEquivalent<T, F> {
    fetch: Box<dyn Fetch<T = T>>,
    map: F,
}

impl<T: 'static + Clone, F: Clone> Clone for MapEquivalent<T, F> {
    fn clone(&self) -> Self {
        Self {
            fetch: self.fetch.clone_boxed(),
            map: self.map.clone(),
        }
    }
}

impl<T, F> FetchBytes for MapEquivalent<T, F> {
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        self.fetch.fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        self.fetch.fetch_data()
    }

    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        self.fetch.as_resolve()
    }

    fn try_unwrap_resolve(self: Box<Self>) -> Option<Arc<dyn Resolve>> {
        self.fetch.try_unwrap_resolve()
    }
}

trait Map1<T>: Clone + FnOnce(T) -> Self::U {
    type U;
}

impl<T, U, F: Clone + FnOnce(T) -> U> Map1<T> for F {
    type U = U;
}

impl<T: 'static + Clone, F: Send + Sync + Map1<T>> Fetch for MapEquivalent<T, F> {
    type T = F::U;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        Box::pin(self.fetch.fetch().map_ok(self.map))
    }

    fn try_unwrap(self: Box<Self>) -> Option<Self::T> {
        let Self { fetch, map } = *self;
        fetch.try_unwrap().map(map)
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
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

impl<T: 'static + Clone, Extra: Clone> Clone for ExtraPoint<T, Extra> {
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
    fn fetch_bytes<'a>(self: Box<Self>) -> FailFuture<'a, ByteNode>
    where
        Self: 'a,
    {
        Box::new(self.point).fetch_bytes()
    }

    fn fetch_data<'a>(self: Box<Self>) -> FailFuture<'a, Vec<u8>>
    where
        Self: 'a,
    {
        Box::new(self.point).fetch_data()
    }
}

impl<T: 'static + FullHash, E: Send + Sync + Clone> Fetch for ExtraPoint<T, E> {
    type T = T;

    fn fetch<'a>(self: Box<Self>) -> FailFuture<'a, Self::T>
    where
        Self: 'a,
    {
        self.point.fetch()
    }

    fn clone_boxed<'a>(&self) -> Box<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a,
        Self::T: Clone,
    {
        Box::new(self.clone())
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
