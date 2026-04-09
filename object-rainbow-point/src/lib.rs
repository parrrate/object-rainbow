use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use futures_util::{TryFutureExt, future::ready};
use object_rainbow::{
    Address, ByteNode, Equivalent, ExtraFor, FailFuture, Fetch, FetchBytes, FullHash, Hash,
    InlineOutput, ListHashes, MaybeHasNiche, Node, ObjectMarker, OptionalHash, Output, Parse,
    ParseAsInline, ParseInline, PointInput, PointVisitor, Resolve, Singular, Size, Tagged, Tags,
    ToOutput, Topological, Traversible,
};

#[cfg(feature = "serde")]
mod point_deserialize;
#[cfg(feature = "point-serialize")]
mod point_serialize;

#[derive(Clone, ParseAsInline)]
struct Extras<Extra>(Extra);

impl<Extra> Deref for Extras<Extra> {
    type Target = Extra;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Extra> ToOutput for Extras<Extra> {
    fn to_output(&self, _: &mut dyn Output) {}
}

impl<Extra> InlineOutput for Extras<Extra> {}

impl<I: PointInput> ParseInline<I> for Extras<I::Extra> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(Self(input.extra().clone()))
    }
}

impl<Extra> Tagged for Extras<Extra> {}
impl<Extra> ListHashes for Extras<Extra> {}
impl<Extra> Topological for Extras<Extra> {}

#[derive(Clone)]
struct ByAddressInner {
    address: Address,
    resolve: Arc<dyn Resolve>,
}

impl FetchBytes for ByAddressInner {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.resolve.resolve(self.address)
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.resolve.resolve_data(self.address)
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.resolve.try_resolve_local(self.address)
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
    fn to_output(&self, output: &mut dyn Output) {
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
}

#[derive(ToOutput, InlineOutput, Tagged, Parse, ParseInline)]
pub struct RawPoint<T, Extra = ()> {
    inner: RawPointInner,
    extra: Extras<Extra>,
    object: ObjectMarker<T>,
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
        Point::from_fetch(self.inner.hash, Arc::new(self))
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
    pub fn from_fetch(hash: Hash, fetch: Arc<dyn Fetch<T = T>>) -> Self {
        Self {
            hash: hash.into(),
            fetch,
        }
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

#[derive(ParseAsInline)]
#[must_use]
pub struct Point<T> {
    hash: OptionalHash,
    fetch: Arc<dyn Fetch<T = T>>,
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
            hash: self.hash.unwrap(),
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
        Self::from_fetch(
            address.hash,
            Arc::new(ByAddress::from_inner(
                ByAddressInner { address, resolve },
                extra,
            )),
        )
    }

    pub fn with_resolve<Extra: 'static + Send + Sync + Clone + ExtraFor<T>>(
        &self,
        resolve: Arc<dyn Resolve>,
        extra: Extra,
    ) -> Self {
        Self::from_address_extra(Address::from_hash(self.hash()), resolve, extra)
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

impl<T: Tagged> Tagged for Point<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut dyn Output) {
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
}

impl<T: Traversible + Clone> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self::from_fetch(object.full_hash(), Arc::new(LocalFetch { object }))
    }

    fn yolo_mut(&mut self) -> bool {
        self.fetch.get().is_some()
            && Arc::get_mut(&mut self.fetch).is_some_and(|fetch| fetch.get_mut().is_some())
    }

    async fn prepare_yolo_fetch(&mut self) -> object_rainbow::Result<()> {
        if !self.yolo_mut() {
            let object = self.fetch.fetch().await?;
            self.fetch = Arc::new(LocalFetch { object });
        }
        Ok(())
    }

    pub async fn fetch_mut(&'_ mut self) -> object_rainbow::Result<PointMut<'_, T>> {
        self.prepare_yolo_fetch().await?;
        let fetch = Arc::get_mut(&mut self.fetch).unwrap();
        assert!(fetch.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            fetch,
        })
    }

    pub async fn fetch_ref(&mut self) -> object_rainbow::Result<&T> {
        self.prepare_yolo_fetch().await?;
        Ok(self.fetch.get().unwrap())
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
        let fetch = Arc::get_mut(&mut self.fetch).unwrap();
        fetch.get_mut_finalize();
        self.hash = fetch.get().unwrap().full_hash().into();
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok()?.fetch.try_unwrap()
    }
}

/// This implementation is the main goal of [`Equivalent`]: we assume transmuting the pointer is
/// safe.
impl<U: 'static + Equivalent<T>, T: 'static> Equivalent<Point<T>> for Point<U> {
    fn into_equivalent(self) -> Point<T> {
        self.map_fetch(|fetch| {
            Arc::new(MapEquivalent {
                fetch,
                map: U::into_equivalent,
            })
        })
    }

    fn from_equivalent(point: Point<T>) -> Self {
        point.map_fetch(|fetch| {
            Arc::new(MapEquivalent {
                fetch,
                map: U::from_equivalent,
            })
        })
    }
}

impl<T> MaybeHasNiche for Point<T> {
    type MnArray = <Hash as MaybeHasNiche>::MnArray;
}

impl<T: FullHash + Default> Point<T> {
    pub fn is_default(&self) -> bool {
        self.hash() == T::default().full_hash()
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

struct LocalFetch<T> {
    object: T,
}

impl<T: Traversible + Clone> Fetch for LocalFetch<T> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(ready(Ok((self.object.clone(), self.object.to_resolve()))))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(ready(Ok(self.object.clone())))
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        Ok(Some((self.object.clone(), self.object.to_resolve())))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        Some(self.object.clone())
    }

    fn get(&self) -> Option<&Self::T> {
        Some(&self.object)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(&mut self.object)
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok().map(|Self { object }| object)
    }
}

impl<T: Traversible + Clone> FetchBytes for LocalFetch<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(Ok((self.object.output(), self.object.to_resolve()))))
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(Ok(self.object.output())))
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        Ok(Some((self.object.output(), self.object.to_resolve())))
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        Some(self.object.output())
    }
}

impl<T: Traversible + Clone> Singular for LocalFetch<T> {
    fn hash(&self) -> Hash {
        self.object.full_hash()
    }
}

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
        self.fetch.get().unwrap()
    }
}

impl<T: FullHash> DerefMut for PointMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fetch.get_mut().unwrap()
    }
}

impl<T: FullHash> Drop for PointMut<'_, T> {
    fn drop(&mut self) {
        self.finalize();
    }
}

impl<'a, T: FullHash> PointMut<'a, T> {
    fn finalize(&mut self) {
        self.fetch.get_mut_finalize();
        *self.hash = self.full_hash().into();
    }
}
