#![deny(unsafe_code)]

extern crate self as object_rainbow;

use std::{
    any::{Any, TypeId},
    cell::Cell,
    convert::Infallible,
    future::ready,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

pub use anyhow::anyhow;
use futures_util::TryFutureExt;
use generic_array::{ArrayLength, GenericArray};
pub use object_rainbow_derive::{
    Enum, Inline, MaybeHasNiche, Object, Parse, ParseAsInline, ParseInline, ReflessInline,
    ReflessObject, Size, Tagged, ToOutput, Topological,
};
use sha2::{Digest, Sha256};
#[doc(hidden)]
pub use typenum;
use typenum::Unsigned;

pub use self::enumkind::Enum;
pub use self::error::{Error, Result};
pub use self::hash::{Hash, OptionalHash};
pub use self::niche::{
    AutoEnumNiche, AutoNiche, HackNiche, MaybeHasNiche, Niche, NicheForUnsized, NoNiche, OneNiche,
    SomeNiche, ZeroNiche, ZeroNoNiche,
};
#[doc(hidden)]
pub use self::niche::{MaybeNiche, MnArray, NicheFoldOrArray, NicheOr};

pub mod enumkind;
mod error;
mod hash;
pub mod hashed;
mod impls;
pub mod length_prefixed;
mod niche;
pub mod numeric;
#[cfg(feature = "serde")]
mod point_deserialize;
#[cfg(feature = "point-serialize")]
mod point_serialize;
mod sha2_const;
pub mod zero_terminated;

/// SHA-256 hash size in bytes.
pub const HASH_SIZE: usize = sha2_const::Sha256::DIGEST_SIZE;

/// Address within a [`PointInput`].
///
/// This was introduced:
/// - to avoid using a [`Hash`]-only map
/// - to differentiate between separate [`Point`]s within a context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ParseAsInline)]
pub struct Address {
    /// Monotonically incremented index. This is not present at all in the actual format.
    pub index: usize,
    /// Only this part is part of the parsed/generated input.
    pub hash: Hash,
}

impl Address {
    /// Construct an address which is invalid within parsing context, but can be used in map-based
    /// [`Resolve`]s.
    pub fn from_hash(hash: Hash) -> Self {
        Self {
            index: usize::MAX,
            hash,
        }
    }
}

impl<I: PointInput> ParseInline<I> for Address {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self {
            index: input.next_index(),
            hash: input.parse_inline()?,
        })
    }
}

/// Fallible future type yielding either `T` or [`Error`].
pub type FailFuture<'a, T> = Pin<Box<dyn 'a + Send + Future<Output = Result<T>>>>;

/// Returned by [`Resolve`] and [`FetchBytes`]. Represents traversal through the object graph.
pub type ByteNode = (Vec<u8>, Arc<dyn Resolve>);

trait FromInner {
    type Inner: 'static + Clone;
    type Extra: 'static + Clone;

    fn from_inner(inner: Self::Inner, extra: Self::Extra) -> Self;
}

/// Trait for contextually using [`Any`]. Can itself be implemented for non-`'static` and `?Sized`
/// types, and is `dyn`-compatible.
pub trait AsAny {
    /// Get a shared RTTI reference.
    fn any_ref(&self) -> &dyn Any
    where
        Self: 'static;
    /// Get an exclusive RTTI reference.
    fn any_mut(&mut self) -> &mut dyn Any
    where
        Self: 'static;
    /// Get an RTTI [`Box`].
    fn any_box(self: Box<Self>) -> Box<dyn Any>
    where
        Self: 'static;
    /// Get an RTTI [`Arc`].
    fn any_arc(self: Arc<Self>) -> Arc<dyn Any>
    where
        Self: 'static;
    /// Get an RTTI [`Arc`] which is also [`Send`].
    fn any_arc_sync(self: Arc<Self>) -> Arc<dyn Send + Sync + Any>
    where
        Self: 'static + Send + Sync;
}

impl<T> AsAny for T {
    fn any_ref(&self) -> &dyn Any
    where
        Self: 'static,
    {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any
    where
        Self: 'static,
    {
        self
    }

    fn any_box(self: Box<Self>) -> Box<dyn Any>
    where
        Self: 'static,
    {
        self
    }

    fn any_arc(self: Arc<Self>) -> Arc<dyn Any>
    where
        Self: 'static,
    {
        self
    }

    fn any_arc_sync(self: Arc<Self>) -> Arc<dyn Send + Sync + Any>
    where
        Self: 'static + Send + Sync,
    {
        self
    }
}

/// Something that resolve [`Address`]es to [`ByteNode`]s.
pub trait Resolve: Send + Sync + AsAny {
    /// Resolve the address. For an [`Object`], this is what gets used as [`PointInput`].
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode>;
    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>>;
    /// Get a dynamic extension for a specific [`Address`].
    fn resolve_extension(&self, address: Address, typeid: TypeId) -> crate::Result<&dyn Any> {
        let _ = address;
        let _ = typeid;
        Err(Error::UnknownExtension)
    }
    /// Get a dynamic extension.
    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        let _ = typeid;
        Err(Error::UnknownExtension)
    }
}

pub trait FetchBytes: AsAny {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode>;
    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>>;
    fn as_inner(&self) -> Option<&dyn Any> {
        None
    }
}

pub trait Fetch: Send + Sync + FetchBytes {
    type T;
    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)>;
    fn fetch(&'_ self) -> FailFuture<'_, Self::T>;
    fn get(&self) -> Option<&Self::T> {
        None
    }
    fn get_mut(&mut self) -> Option<&mut Self::T> {
        None
    }
    fn get_mut_finalize(&mut self) {}
}

trait FetchBytesExt: FetchBytes {
    fn inner_cast<T: FromInner>(&self, extra: &T::Extra) -> Option<T> {
        self.as_inner()?
            .downcast_ref()
            .cloned()
            .map(|inner| T::from_inner(inner, extra.clone()))
    }
}

impl<T: ?Sized + FetchBytes> FetchBytesExt for T {}

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

impl<I: PointInput> ParseInline<I> for RawPointInner {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self::from_address(input.parse_inline()?, input.resolve()))
    }
}

impl Tagged for RawPointInner {}

impl Singular for RawPointInner {
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl<T, Extra: Send + Sync> Singular for RawPoint<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

#[derive(ToOutput, Topological, Parse, ParseInline)]
pub struct ObjectMarker<T: ?Sized> {
    object: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Clone for ObjectMarker<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for ObjectMarker<T> {}

impl<T: ?Sized> Default for ObjectMarker<T> {
    fn default() -> Self {
        Self {
            object: Default::default(),
        }
    }
}

impl<T: ?Sized + Tagged> Tagged for ObjectMarker<T> {}
impl<T: ?Sized + 'static + Tagged, Extra: 'static> Object<Extra> for ObjectMarker<T> {}
impl<T: ?Sized + 'static + Tagged, Extra: 'static> Inline<Extra> for ObjectMarker<T> {}

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

impl<I: PointInput> ParseInline<I> for Extras<I::Extra> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self(input.extra().clone()))
    }
}

impl<Extra> Tagged for Extras<Extra> {}
impl<Extra> Topological for Extras<Extra> {}
impl<Extra: 'static + Send + Sync + Clone> Object<Extra> for Extras<Extra> {}
impl<Extra: 'static + Send + Sync + Clone> Inline<Extra> for Extras<Extra> {}

#[derive(ToOutput, Tagged, Parse, ParseInline)]
pub struct RawPoint<T = Infallible, Extra = ()> {
    inner: RawPointInner,
    extra: Extras<Extra>,
    object: ObjectMarker<T>,
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Topological for RawPoint<T, Extra> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(&self.clone().point());
    }

    fn point_count(&self) -> usize {
        1
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Object<Extra> for RawPoint<T, Extra> {}
impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Inline<Extra> for RawPoint<T, Extra> {}

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

impl<T, Extra: 'static + Clone> RawPoint<T, Extra> {
    pub fn cast<U>(self) -> RawPoint<U, Extra> {
        self.inner.cast(self.extra.0)
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> RawPoint<T, Extra> {
    pub fn point(self) -> Point<T> {
        Point::from_fetch(self.inner.hash, Arc::new(self))
    }
}

impl FetchBytes for RawPointInner {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.fetch.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.fetch.fetch_data()
    }
}

impl<T, Extra> FetchBytes for RawPoint<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.inner.fetch_data()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }
}

impl<T: Object<Extra>, Extra: Send + Sync> Fetch for RawPoint<T, Extra> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(async {
            let (data, resolve) = self.inner.fetch.fetch_bytes().await?;
            let object = T::parse_slice_extra(&data, &resolve, &self.extra)?;
            if self.inner.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok((object, resolve))
            }
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (data, resolve) = self.inner.fetch.fetch_bytes().await?;
            let object = T::parse_slice_extra(&data, &resolve, &self.extra)?;
            if self.inner.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok(object)
            }
        })
    }
}

impl<T> Point<T> {
    pub fn extract_resolve<R: Any>(&self) -> Option<(&Address, &R)> {
        let ByAddressInner { address, resolve } =
            self.fetch.as_inner()?.downcast_ref::<ByAddressInner>()?;
        let resolve = resolve.as_ref().any_ref().downcast_ref::<R>()?;
        Some((address, resolve))
    }
}

#[derive(ParseAsInline)]
#[must_use]
pub struct Point<T> {
    hash: OptionalHash,
    fetch: Arc<dyn Fetch<T = T>>,
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

impl<T> Size for Point<T> {
    const SIZE: usize = HASH_SIZE;
    type Size = typenum::generic_const_mappings::U<HASH_SIZE>;
}

impl<T: Object> Point<T> {
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self::from_address_extra(address, resolve, ())
    }
}

impl<T> Point<T> {
    pub fn from_address_extra<Extra: 'static + Send + Sync + Clone>(
        address: Address,
        resolve: Arc<dyn Resolve>,
        extra: Extra,
    ) -> Self
    where
        T: Object<Extra>,
    {
        Self::from_fetch(
            address.hash,
            Arc::new(ByAddress::from_inner(
                ByAddressInner { address, resolve },
                extra,
            )),
        )
    }

    pub fn with_resolve<Extra: 'static + Send + Sync + Clone>(
        &self,
        resolve: Arc<dyn Resolve>,
        extra: Extra,
    ) -> Self
    where
        T: Object<Extra>,
    {
        Self::from_address_extra(Address::from_hash(self.hash()), resolve, extra)
    }
}

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

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }
}

impl<T: Object<Extra>, Extra: Send + Sync> Fetch for ByAddress<T, Extra> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(async {
            let (data, resolve) = self.fetch_bytes().await?;
            let object = T::parse_slice_extra(&data, &resolve, &self.extra)?;
            if self.inner.address.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok((object, resolve))
            }
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async {
            let (data, resolve) = self.fetch_bytes().await?;
            let object = T::parse_slice_extra(&data, &resolve, &self.extra)?;
            if self.inner.address.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok(object)
            }
        })
    }
}

pub trait PointVisitor {
    fn visit<T: Traversible>(&mut self, point: &Point<T>);
}

struct HashVisitor<F>(F);

impl<F: FnMut(Hash)> PointVisitor for HashVisitor<F> {
    fn visit<T: Traversible>(&mut self, point: &Point<T>) {
        self.0(point.hash());
    }
}

pub struct ReflessInput<'d> {
    data: Option<&'d [u8]>,
}

pub struct Input<'d, Extra = ()> {
    refless: ReflessInput<'d>,
    resolve: &'d Arc<dyn Resolve>,
    index: &'d Cell<usize>,
    extra: &'d Extra,
}

impl<'a, Extra> Deref for Input<'a, Extra> {
    type Target = ReflessInput<'a>;

    fn deref(&self) -> &Self::Target {
        &self.refless
    }
}

impl<Extra> DerefMut for Input<'_, Extra> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.refless
    }
}

impl<'a, Extra> Input<'a, Extra> {
    pub fn replace_extra<E>(self, extra: &'a E) -> Input<'a, E> {
        Input {
            refless: self.refless,
            resolve: self.resolve,
            index: self.index,
            extra,
        }
    }
}

impl<'a> ReflessInput<'a> {
    fn data(&self) -> crate::Result<&'a [u8]> {
        self.data.ok_or(Error::EndOfInput)
    }

    fn make_error<T>(&mut self, e: crate::Error) -> crate::Result<T> {
        self.data = None;
        Err(e)
    }

    fn end_of_input<T>(&mut self) -> crate::Result<T> {
        self.make_error(Error::EndOfInput)
    }
}

impl<'d> ParseInput for ReflessInput<'d> {
    type Data = &'d [u8];

    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a,
    {
        match self.data()?.split_first_chunk() {
            Some((chunk, data)) => {
                self.data = Some(data);
                Ok(chunk)
            }
            None => self.end_of_input(),
        }
    }

    fn parse_n(&mut self, n: usize) -> crate::Result<Self::Data> {
        match self.data()?.split_at_checked(n) {
            Some((chunk, data)) => {
                self.data = Some(data);
                Ok(chunk)
            }
            None => self.end_of_input(),
        }
    }

    fn parse_until_zero(&mut self) -> crate::Result<Self::Data> {
        let data = self.data()?;
        match data.iter().enumerate().find(|(_, x)| **x == 0) {
            Some((at, _)) => {
                let (chunk, data) = data.split_at(at);
                self.data = Some(&data[1..]);
                Ok(chunk)
            }
            None => self.end_of_input(),
        }
    }

    fn reparse<T: Parse<Self>>(&mut self, data: Self::Data) -> crate::Result<T> {
        let input = Self { data: Some(data) };
        T::parse(input)
    }

    fn parse_all(self) -> crate::Result<Self::Data> {
        self.data()
    }

    fn empty(self) -> crate::Result<()> {
        if self.data()?.is_empty() {
            Ok(())
        } else {
            Err(Error::ExtraInputLeft)
        }
    }

    fn non_empty(self) -> crate::Result<Option<Self>> {
        Ok(if self.data()?.is_empty() {
            None
        } else {
            Some(self)
        })
    }
}

impl<'d, Extra> ParseInput for Input<'d, Extra> {
    type Data = &'d [u8];

    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a,
    {
        (**self).parse_chunk()
    }

    fn parse_n(&mut self, n: usize) -> crate::Result<Self::Data> {
        (**self).parse_n(n)
    }

    fn parse_until_zero(&mut self) -> crate::Result<Self::Data> {
        (**self).parse_until_zero()
    }

    fn reparse<T: Parse<Self>>(&mut self, data: Self::Data) -> crate::Result<T> {
        let input = Self {
            refless: ReflessInput { data: Some(data) },
            resolve: self.resolve,
            index: self.index,
            extra: self.extra,
        };
        T::parse(input)
    }

    fn parse_all(self) -> crate::Result<Self::Data> {
        self.refless.parse_all()
    }

    fn empty(self) -> crate::Result<()> {
        self.refless.empty()
    }

    fn non_empty(mut self) -> crate::Result<Option<Self>> {
        self.refless = match self.refless.non_empty()? {
            Some(refless) => refless,
            None => return Ok(None),
        };
        Ok(Some(self))
    }
}

impl<'d, Extra: 'static + Clone> PointInput for Input<'d, Extra> {
    type Extra = Extra;
    type WithExtra<E: 'static + Clone> = Input<'d, E>;

    fn next_index(&mut self) -> usize {
        let index = self.index.get();
        self.index.set(index + 1);
        index
    }

    fn resolve_arc_ref(&self) -> &Arc<dyn Resolve> {
        self.resolve
    }

    fn extra(&self) -> &Self::Extra {
        self.extra
    }

    fn map_extra<E: 'static + Clone>(
        self,
        f: impl FnOnce(&Self::Extra) -> &E,
    ) -> Self::WithExtra<E> {
        let Self {
            refless,
            resolve,
            index,
            extra,
        } = self;
        Input {
            refless,
            resolve,
            index,
            extra: f(extra),
        }
    }
}

pub trait ToOutput {
    fn to_output(&self, output: &mut dyn Output);

    fn data_hash(&self) -> Hash {
        let mut output = HashOutput::default();
        self.to_output(&mut output);
        output.hash()
    }

    fn slice_to_output(slice: &[Self], output: &mut dyn Output)
    where
        Self: Sized,
    {
        slice.iter_to_output(output);
    }

    fn output<T: Output + Default>(&self) -> T {
        let mut output = T::default();
        self.to_output(&mut output);
        output
    }

    fn vec(&self) -> Vec<u8> {
        self.output()
    }
}

#[derive(Default)]
struct CountVisitor {
    count: usize,
}

impl PointVisitor for CountVisitor {
    fn visit<T: Traversible>(&mut self, _: &Point<T>) {
        self.count += 1;
    }
}

pub trait Topological {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        let _ = visitor;
    }

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        self.accept_points(&mut HashVisitor(|hash| hasher.update(hash)));
        Hash::from_sha256(hasher.finalize().into())
    }

    fn point_count(&self) -> usize {
        let mut visitor = CountVisitor::default();
        self.accept_points(&mut visitor);
        visitor.count
    }

    fn topology(&self) -> TopoVec {
        let mut topology = TopoVec::with_capacity(self.point_count());
        self.accept_points(&mut topology);
        topology
    }
}

pub trait Tagged {
    const TAGS: Tags = Tags(&[], &[]);

    const HASH: Hash =
        const { Hash::from_sha256(Self::TAGS.const_hash(sha2_const::Sha256::new()).finalize()) };
}

pub trait ParseSlice: for<'a> Parse<Input<'a>> {
    fn parse_slice(data: &[u8], resolve: &Arc<dyn Resolve>) -> crate::Result<Self> {
        Self::parse_slice_extra(data, resolve, &())
    }
}

impl<T: for<'a> Parse<Input<'a>>> ParseSlice for T {}

pub trait ParseSliceExtra<Extra>: for<'a> Parse<Input<'a, Extra>> {
    fn parse_slice_extra(
        data: &[u8],
        resolve: &Arc<dyn Resolve>,
        extra: &Extra,
    ) -> crate::Result<Self> {
        let input = Input {
            refless: ReflessInput { data: Some(data) },
            resolve,
            index: &Cell::new(0),
            extra,
        };
        let object = Self::parse(input)?;
        Ok(object)
    }
}

impl<T: for<'a> Parse<Input<'a, Extra>>, Extra> ParseSliceExtra<Extra> for T {}

#[derive(ToOutput)]
pub struct ObjectHashes {
    pub tags: Hash,
    pub topology: Hash,
    pub data: Hash,
}

pub trait FullHash: ToOutput + Topological + Tagged {
    fn hashes(&self) -> ObjectHashes {
        ObjectHashes {
            tags: Self::HASH,
            topology: self.topology_hash(),
            data: self.data_hash(),
        }
    }

    fn full_hash(&self) -> Hash {
        self.hashes().data_hash()
    }
}

impl<T: ?Sized + ToOutput + Topological + Tagged> FullHash for T {}

pub trait Traversible: 'static + Sized + Send + Sync + FullHash {
    fn to_resolve(&self) -> Arc<dyn Resolve> {
        Arc::new(ByTopology {
            topology: self.topology(),
        })
    }

    fn point(self) -> Point<Self>
    where
        Self: Clone,
    {
        Point::from_object(self)
    }
}

impl<T: 'static + Send + Sync + FullHash> Traversible for T {}

pub trait Object<Extra = ()>: Traversible + for<'a> Parse<Input<'a, Extra>> {}

pub struct Tags(pub &'static [&'static str], pub &'static [&'static Self]);

impl Tags {
    const fn const_hash(&self, mut hasher: sha2_const::Sha256) -> sha2_const::Sha256 {
        {
            let mut i = 0;
            while i < self.0.len() {
                hasher = hasher.update(self.0[i].as_bytes());
                i += 1;
            }
        }
        {
            let mut i = 0;
            while i < self.1.len() {
                hasher = self.1[i].const_hash(hasher);
                i += 1;
            }
        }
        hasher
    }
}

pub trait Inline<Extra = ()>: Object<Extra> + for<'a> ParseInline<Input<'a, Extra>> {}

impl<T: Traversible> Topological for Point<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }

    fn point_count(&self) -> usize {
        1
    }
}

impl<T: Object<I::Extra>, I: PointInput<Extra: Send + Sync>> ParseInline<I> for Point<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
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

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Object<Extra> for Point<T> {}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.hash().to_output(output);
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Inline<Extra> for Point<T> {}

pub trait Topology: Send + Sync {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Arc<dyn Singular>>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Singular: Send + Sync + FetchBytes {
    fn hash(&self) -> Hash;
}

pub type TopoVec = Vec<Arc<dyn Singular>>;

impl PointVisitor for TopoVec {
    fn visit<T: Traversible>(&mut self, point: &Point<T>) {
        self.push(Arc::new(point.clone()));
    }
}

impl<T> FetchBytes for Point<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.fetch.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.fetch.fetch_data()
    }
}

impl<T> Singular for Point<T> {
    fn hash(&self) -> Hash {
        self.hash.unwrap()
    }
}

impl Topology for TopoVec {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, index: usize) -> Option<&Arc<dyn Singular>> {
        (**self).get(index)
    }
}

pub trait ParseSliceRefless: for<'a> Parse<ReflessInput<'a>> {
    fn parse_slice_refless(data: &[u8]) -> crate::Result<Self> {
        let input = ReflessInput { data: Some(data) };
        let object = Self::parse(input)?;
        Ok(object)
    }
}

impl<T: for<'a> Parse<ReflessInput<'a>>> ParseSliceRefless for T {}

pub trait ReflessObject:
    'static + Sized + Send + Sync + ToOutput + Tagged + for<'a> Parse<ReflessInput<'a>>
{
}

pub trait ReflessInline: ReflessObject + for<'a> ParseInline<ReflessInput<'a>> {}

pub trait Output {
    fn write(&mut self, data: &[u8]);
}

impl Output for Vec<u8> {
    fn write(&mut self, data: &[u8]) {
        self.extend_from_slice(data);
    }
}

#[derive(Default)]
struct HashOutput {
    hasher: Sha256,
    at: usize,
}

impl Output for HashOutput {
    fn write(&mut self, data: &[u8]) {
        self.hasher.update(data);
        self.at += data.len();
    }
}

impl HashOutput {
    fn hash(self) -> Hash {
        Hash::from_sha256(self.hasher.finalize().into())
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

impl<T> Point<T> {
    pub fn get(&self) -> Option<&T> {
        self.fetch.get()
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

    async fn prepare_yolo_fetch(&mut self) -> crate::Result<()> {
        if !self.yolo_mut() {
            let object = self.fetch.fetch().await?;
            self.fetch = Arc::new(LocalFetch { object });
        }
        Ok(())
    }

    pub async fn fetch_mut(&'_ mut self) -> crate::Result<PointMut<'_, T>> {
        self.prepare_yolo_fetch().await?;
        let origin = Arc::get_mut(&mut self.fetch).unwrap();
        assert!(origin.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            fetch: origin,
        })
    }

    pub async fn fetch_ref(&mut self) -> crate::Result<&T> {
        self.prepare_yolo_fetch().await?;
        Ok(self.fetch.get().unwrap())
    }
}

struct LocalFetch<T> {
    object: T,
}

impl<T> Deref for LocalFetch<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T> DerefMut for LocalFetch<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<T: Traversible + Clone> Fetch for LocalFetch<T> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(ready(Ok((self.object.clone(), self.object.to_resolve()))))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(ready(Ok(self.object.clone())))
    }

    fn get(&self) -> Option<&Self::T> {
        Some(self)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(self)
    }
}

impl<T: Traversible + Clone> FetchBytes for LocalFetch<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(Ok((self.object.output(), self.object.to_resolve()))))
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(ready(Ok(self.object.output())))
    }
}

struct ByTopology {
    topology: TopoVec,
}

impl ByTopology {
    fn try_resolve(&'_ self, address: Address) -> Result<FailFuture<'_, ByteNode>> {
        let point = self
            .topology
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        if point.hash() != address.hash {
            Err(Error::ResolutionMismatch)
        } else {
            Ok(point.fetch_bytes())
        }
    }

    fn try_resolve_data(&'_ self, address: Address) -> Result<FailFuture<'_, Vec<u8>>> {
        let point = self
            .topology
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        if point.hash() != address.hash {
            Err(Error::ResolutionMismatch)
        } else {
            Ok(point.fetch_data())
        }
    }
}

impl Resolve for ByTopology {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        self.try_resolve(address)
            .map_err(Err)
            .map_err(ready)
            .map_err(Box::pin)
            .unwrap_or_else(|x| x)
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        self.try_resolve_data(address)
            .map_err(Err)
            .map_err(ready)
            .map_err(Box::pin)
            .unwrap_or_else(|x| x)
    }
}

impl<T: FullHash> Fetch for Point<T> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        self.fetch.fetch_full()
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        self.fetch.fetch()
    }

    fn get(&self) -> Option<&Self::T> {
        self.fetch.get()
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        self.hash.clear();
        Arc::get_mut(&mut self.fetch)?.get_mut()
    }

    fn get_mut_finalize(&mut self) {
        let origin = Arc::get_mut(&mut self.fetch).unwrap();
        origin.get_mut_finalize();
        self.hash = origin.get().unwrap().full_hash().into();
    }
}

pub trait Size {
    const SIZE: usize = <Self::Size as Unsigned>::USIZE;
    type Size: Unsigned;
}

pub trait SizeExt: Size<Size: ArrayLength> + ToOutput {
    fn to_array(&self) -> GenericArray<u8, Self::Size> {
        let mut array = GenericArray::default();
        let mut output = ArrayOutput {
            data: &mut array,
            offset: 0,
        };
        self.to_output(&mut output);
        output.finalize();
        array
    }
}

impl<T: Size<Size: ArrayLength> + ToOutput> SizeExt for T {}

struct ArrayOutput<'a> {
    data: &'a mut [u8],
    offset: usize,
}

impl ArrayOutput<'_> {
    fn finalize(self) {
        assert_eq!(self.offset, self.data.len());
    }
}

impl Output for ArrayOutput<'_> {
    fn write(&mut self, data: &[u8]) {
        self.data[self.offset..][..data.len()].copy_from_slice(data);
        self.offset += data.len();
    }
}

trait RainbowIterator: Sized + IntoIterator {
    fn iter_to_output(self, output: &mut dyn Output)
    where
        Self::Item: ToOutput,
    {
        self.into_iter().for_each(|item| item.to_output(output));
    }

    fn iter_accept_points(self, visitor: &mut impl PointVisitor)
    where
        Self::Item: Topological,
    {
        self.into_iter()
            .for_each(|item| item.accept_points(visitor));
    }
}

pub trait ParseInput: Sized {
    type Data: AsRef<[u8]> + Deref<Target = [u8]> + Into<Vec<u8>> + Copy;
    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a;
    fn parse_n(&mut self, n: usize) -> crate::Result<Self::Data>;
    fn parse_until_zero(&mut self) -> crate::Result<Self::Data>;
    fn parse_n_compare(&mut self, n: usize, c: &[u8]) -> crate::Result<Option<Self::Data>> {
        let data = self.parse_n(n)?;
        if *data == *c {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }
    fn reparse<T: Parse<Self>>(&mut self, data: Self::Data) -> crate::Result<T>;
    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T> {
        let data = self.parse_n(n)?;
        self.reparse(data)
    }
    fn parse_zero_terminated<T: Parse<Self>>(&mut self) -> crate::Result<T> {
        let data = self.parse_until_zero()?;
        self.reparse(data)
    }
    fn parse_compare<T: Parse<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>> {
        self.parse_n_compare(n, c)?
            .map(|data| self.reparse(data))
            .transpose()
    }
    fn parse_all(self) -> crate::Result<Self::Data>;
    fn empty(self) -> crate::Result<()>;
    fn non_empty(self) -> crate::Result<Option<Self>>;

    fn consume(self, f: impl FnMut(&mut Self) -> crate::Result<()>) -> crate::Result<()> {
        self.collect(f)
    }

    fn parse_collect<T: ParseInline<Self>, B: FromIterator<T>>(self) -> crate::Result<B> {
        self.collect(|input| input.parse_inline())
    }

    fn collect<T, B: FromIterator<T>>(
        self,
        f: impl FnMut(&mut Self) -> crate::Result<T>,
    ) -> crate::Result<B> {
        self.iter(f).collect()
    }

    fn iter<T>(
        self,
        mut f: impl FnMut(&mut Self) -> crate::Result<T>,
    ) -> impl Iterator<Item = crate::Result<T>> {
        let mut state = Some(self);
        std::iter::from_fn(move || {
            let mut input = match state.take()?.non_empty() {
                Ok(input) => input?,
                Err(e) => return Some(Err(e)),
            };
            let item = f(&mut input);
            state = Some(input);
            Some(item)
        })
    }

    fn parse_inline<T: ParseInline<Self>>(&mut self) -> crate::Result<T> {
        T::parse_inline(self)
    }

    fn parse<T: Parse<Self>>(self) -> crate::Result<T> {
        T::parse(self)
    }

    fn parse_vec<T: ParseInline<Self>>(self) -> crate::Result<Vec<T>> {
        T::parse_vec(self)
    }
}

pub trait PointInput: ParseInput {
    type Extra: 'static + Clone;
    type WithExtra<E: 'static + Clone>: PointInput<Extra = E, WithExtra<Self::Extra> = Self>;
    fn next_index(&mut self) -> usize;
    fn resolve_arc_ref(&self) -> &Arc<dyn Resolve>;
    fn resolve(&self) -> Arc<dyn Resolve> {
        self.resolve_arc_ref().clone()
    }
    fn resolve_ref(&self) -> &dyn Resolve {
        self.resolve_arc_ref().as_ref()
    }
    fn extension<T: Any>(&self) -> crate::Result<&T> {
        self.resolve_ref()
            .extension(TypeId::of::<T>())?
            .downcast_ref()
            .ok_or(Error::ExtensionType)
    }
    fn extra(&self) -> &Self::Extra;
    fn map_extra<E: 'static + Clone>(
        self,
        f: impl FnOnce(&Self::Extra) -> &E,
    ) -> Self::WithExtra<E>;
}

impl<T: Sized + IntoIterator> RainbowIterator for T {}

pub trait Parse<I: ParseInput>: Sized {
    fn parse(input: I) -> crate::Result<Self>;
}

pub trait ParseInline<I: ParseInput>: Parse<I> {
    fn parse_inline(input: &mut I) -> crate::Result<Self>;
    fn parse_as_inline(mut input: I) -> crate::Result<Self> {
        let object = Self::parse_inline(&mut input)?;
        input.empty()?;
        Ok(object)
    }
    fn parse_vec(input: I) -> crate::Result<Vec<Self>> {
        input.parse_collect()
    }
}

/// Implemented if both types have the exact same layout.
/// This implies having the same [`MaybeHasNiche::MnArray`].
///
/// This is represented as two-way conversion for two reasons:
/// - to highlight that the conversion is actual equivalence
/// - to increase flexibility (mostly to go around the orphan rule)
pub trait Equivalent<T>: Sized {
    /// Inverse of [`Equivalent::from_equivalent`].
    fn into_equivalent(self) -> T;
    /// Inverse of [`Equivalent::into_equivalent`].
    fn from_equivalent(object: T) -> Self;
}

/// This implementation is the main goal of [`Equivalent`]: we assume transmuting the pointer is
/// safe.
impl<U: 'static + Equivalent<T>, T: 'static> Equivalent<Point<T>> for Point<U> {
    fn into_equivalent(self) -> Point<T> {
        self.map_fetch(|origin| {
            Arc::new(MapEquivalent {
                origin,
                map: U::into_equivalent,
            })
        })
    }

    fn from_equivalent(point: Point<T>) -> Self {
        point.map_fetch(|origin| {
            Arc::new(MapEquivalent {
                origin,
                map: U::from_equivalent,
            })
        })
    }
}

struct MapEquivalent<T, F> {
    origin: Arc<dyn Fetch<T = T>>,
    map: F,
}

impl<T, F> FetchBytes for MapEquivalent<T, F> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.origin.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.origin.fetch_data()
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

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
    }
}

impl<T: 'static + ToOutput> Point<T> {
    pub fn map<U>(self, map: impl 'static + Send + Sync + Fn(T) -> U) -> Point<U> {
        self.map_fetch(|origin| Arc::new(Map { origin, map }))
    }
}

struct Map<T, F> {
    origin: Arc<dyn Fetch<T = T>>,
    map: F,
}

impl<T: ToOutput, F> FetchBytes for Map<T, F> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| (x.output(), r)))
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(self.origin.fetch().map_ok(|x| x.output()))
    }
}

impl<T: ToOutput, F: Send + Sync + Map1<T>> Fetch for Map<T, F> {
    type T = F::U;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
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

#[test]
fn options() {
    type T0 = ();
    type T1 = Option<T0>;
    type T2 = Option<T1>;
    type T3 = Option<T2>;
    type T4 = Option<T3>;
    type T5 = Option<T4>;
    assert_eq!(T0::SIZE, 0);
    assert_eq!(T1::SIZE, 1);
    assert_eq!(T2::SIZE, 1);
    assert_eq!(T3::SIZE, 1);
    assert_eq!(T4::SIZE, 1);
    assert_eq!(T5::SIZE, 1);
    assert_eq!(Some(Some(Some(()))).vec(), [0]);
    assert_eq!(Some(Some(None::<()>)).vec(), [1]);
    assert_eq!(Some(None::<Option<()>>).vec(), [2]);
    assert_eq!(None::<Option<Option<()>>>.vec(), [3]);

    assert_eq!(false.vec(), [0]);
    assert_eq!(true.vec(), [1]);
    assert_eq!(Some(false).vec(), [0]);
    assert_eq!(Some(true).vec(), [1]);
    assert_eq!(None::<bool>.vec(), [2]);
    assert_eq!(Some(Some(false)).vec(), [0]);
    assert_eq!(Some(Some(true)).vec(), [1]);
    assert_eq!(Some(None::<bool>).vec(), [2]);
    assert_eq!(None::<Option<bool>>.vec(), [3]);
    assert_eq!(Some(Some(Some(false))).vec(), [0]);
    assert_eq!(Some(Some(Some(true))).vec(), [1]);
    assert_eq!(Some(Some(None::<bool>)).vec(), [2]);
    assert_eq!(Some(None::<Option<bool>>).vec(), [3]);
    assert_eq!(None::<Option<Option<bool>>>.vec(), [4]);
    assert_eq!(Option::<Point<()>>::SIZE, HASH_SIZE);
    assert_eq!(Some(()).vec(), [0]);
    assert_eq!(Some(((), ())).vec(), [0]);
    assert_eq!(Some(((), true)).vec(), [1]);
    assert_eq!(Some((true, true)).vec(), [1, 1]);
    assert_eq!(Some((Some(true), true)).vec(), [1, 1]);
    assert_eq!(Some((None::<bool>, true)).vec(), [2, 1]);
    assert_eq!(Some((true, None::<bool>)).vec(), [1, 2]);
    assert_eq!(None::<(Option<bool>, bool)>.vec(), [3, 2]);
    assert_eq!(None::<(bool, Option<bool>)>.vec(), [2, 3]);
    assert_eq!(Some(Some((Some(true), Some(true)))).vec(), [1, 1],);
    assert_eq!(Option::<Point<()>>::SIZE, HASH_SIZE);
    assert_eq!(Option::<Option<Point<()>>>::SIZE, HASH_SIZE);
    assert_eq!(Option::<Option<Option<Point<()>>>>::SIZE, HASH_SIZE);
}
