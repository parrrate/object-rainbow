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
pub use self::hash::{Hash, OptionalHash};
pub use self::niche::{
    AutoEnumNiche, AutoNiche, HackNiche, MaybeHasNiche, Niche, NicheForUnsized, NoNiche, OneNiche,
    SomeNiche, ZeroNiche, ZeroNoNiche,
};
#[doc(hidden)]
pub use self::niche::{MaybeNiche, MnArray, NicheFoldOrArray, NicheOr};

pub mod enumkind;
mod hash;
pub mod hashed;
mod impls;
pub mod length_prefixed;
mod niche;
pub mod numeric;
mod sha2_const;

#[macro_export]
/// Construct [`Error::Parse`].
macro_rules! error_parse {
    ($($t:tt)*) => {
        $crate::Error::Parse($crate::anyhow!($($t)*))
    };
}

#[macro_export]
/// Construct [`Error::Fetch`].
macro_rules! error_fetch {
    ($($t:tt)*) => {
        $crate::Error::Fetch($crate::anyhow!($($t)*))
    };
}

/// Errors encountered during fetching an object. Mostly related to parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Arbitrary parsing error.
    #[error(transparent)]
    Parse(anyhow::Error),
    /// Arbitrary fetching error.
    #[error(transparent)]
    Fetch(anyhow::Error),
    /// Data left after an [`Inline`] got parsed.
    #[error("extra input left")]
    ExtraInputLeft,
    /// EOF.
    #[error("end of input")]
    EndOfInput,
    /// Overran [`PointInput`]'s [`Address`] vector.
    #[error("address index out of bounds")]
    AddressOutOfBounds,
    /// [`Address::hash`] doesn't match what [`Resolve`] returned.
    #[error("hash resolution mismatch")]
    ResolutionMismatch,
    /// [`FullHash::full_hash`] doesn't match [`Singular::hash`].
    #[error("data hash mismatch")]
    DataMismatch,
    /// Discriminant out of range for an [`Enum`].
    #[error("discriminant overflow")]
    DiscriminantOverflow,
    /// Unepxected zero for a non-zero value.
    #[error("zero")]
    Zero,
    /// Value out of bounds for a certain type.
    #[error("out of bounds")]
    OutOfBounds,
    /// Current architecture (32-bit) is unable to handle lengths of this size.
    #[error("length out of bounds")]
    UnsupportedLength,
    /// Not UTF-8.
    #[error(transparent)]
    Utf8(std::string::FromUtf8Error),
    /// [`Resolve::extension`] (or related things) were unable to resolve the extension.
    #[error("unknown extension")]
    UnknownExtension,
    /// Extension type didn't match what we asked for. This might be turned into panic later.
    #[error("wrong extension type")]
    ExtensionType,
}

impl Error {
    /// Construct [`Error::Parse`] from another error.
    pub fn parse(e: impl Into<anyhow::Error>) -> Self {
        Self::Parse(e.into())
    }

    /// Construct [`Error::Fetch`] from another error.
    pub fn fetch(e: impl Into<anyhow::Error>) -> Self {
        Self::Fetch(e.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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
    /// Self-identification of the resolver type.
    fn name(&self) -> &str;
}

pub trait FetchBytes: AsAny {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode>;
    fn as_inner(&self) -> Option<&dyn Any> {
        None
    }
    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        let _ = typeid;
        Err(Error::UnknownExtension)
    }
}

pub trait Fetch: Send + Sync + FetchBytes {
    type T;
    type Extra;
    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)>;
    fn fetch(&'_ self) -> FailFuture<'_, Self::T>;
    fn get(&self) -> Option<&Self::T> {
        None
    }
    fn get_mut(&mut self) -> Option<&mut Self::T> {
        None
    }
    fn get_mut_finalize(&mut self) {}
    fn extra(&self) -> &Self::Extra;
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
    origin: Arc<dyn Send + Sync + FetchBytes>,
}

impl RawPointInner {
    pub fn cast<T, Extra: 'static + Clone>(self, extra: Extra) -> RawPoint<T, Extra> {
        RawPoint::from_inner(self, extra)
    }
}

impl RawPointInner {
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self {
            hash: address.hash,
            origin: Arc::new(ByAddressInner { address, resolve }),
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

impl<T, Extra: 'static + Send + Sync> Singular for RawPoint<T, Extra> {
    fn hash(&self) -> Hash {
        self.inner.hash()
    }
}

#[derive(ToOutput, Topological, Parse, ParseInline)]
struct ObjectMarker<T: ?Sized> {
    object: PhantomData<fn() -> T>,
}

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

#[derive(ToOutput, Tagged, Parse, ParseInline)]
pub struct RawPoint<T = Infallible, Extra = ()> {
    inner: RawPointInner,
    extra: Extras<Extra>,
    object: ObjectMarker<T>,
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Topological<Extra>
    for RawPoint<T, Extra>
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<T, Extra: 'static + Clone> Point<T, Extra> {
    pub fn raw(self) -> RawPoint<T, Extra> {
        {
            if let Some(raw) = self.origin.inner_cast(self.origin.extra()) {
                return raw;
            }
        }
        let extra = self.origin.extra().clone();
        RawPointInner {
            hash: self.hash.unwrap(),
            origin: self.origin,
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
    pub fn point(self) -> Point<T, Extra> {
        Point::from_origin(self.inner.hash, Arc::new(self))
    }
}

impl FetchBytes for RawPointInner {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.origin.fetch_bytes()
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.origin.extension(typeid)
    }
}

impl<T, Extra: 'static> FetchBytes for RawPoint<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.inner.extension(typeid)
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync> Fetch for RawPoint<T, Extra> {
    type T = T;
    type Extra = Extra;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(async {
            let (data, resolve) = self.inner.origin.fetch_bytes().await?;
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
            let (data, resolve) = self.inner.origin.fetch_bytes().await?;
            let object = T::parse_slice_extra(&data, &resolve, &self.extra)?;
            if self.inner.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok(object)
            }
        })
    }

    fn extra(&self) -> &Self::Extra {
        &self.extra
    }
}

impl<T, Extra: 'static> Point<T, Extra> {
    pub fn extract_resolve<R: Any>(&self) -> Option<(&Address, &R)> {
        let ByAddressInner { address, resolve } =
            self.origin.as_inner()?.downcast_ref::<ByAddressInner>()?;
        let resolve = resolve.as_ref().any_ref().downcast_ref::<R>()?;
        Some((address, resolve))
    }
}

#[derive(ParseAsInline)]
#[must_use]
pub struct Point<T, Extra = ()> {
    hash: OptionalHash,
    origin: Arc<dyn Fetch<T = T, Extra = Extra>>,
}

impl<T, Extra> PartialOrd for Point<T, Extra> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, Extra> Ord for Point<T, Extra> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash().cmp(&other.hash())
    }
}

impl<T, Extra> Eq for Point<T, Extra> {}

impl<T, Extra> PartialEq for Point<T, Extra> {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl<T, Extra> Clone for Point<T, Extra> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            origin: self.origin.clone(),
        }
    }
}

impl<T, Extra> Point<T, Extra> {
    fn from_origin(hash: Hash, origin: Arc<dyn Fetch<T = T, Extra = Extra>>) -> Self {
        Self {
            hash: hash.into(),
            origin,
        }
    }

    fn map_origin<U>(
        self,
        f: impl FnOnce(Arc<dyn Fetch<T = T, Extra = Extra>>) -> Arc<dyn Fetch<T = U, Extra = Extra>>,
    ) -> Point<U, Extra> {
        Point {
            hash: self.hash,
            origin: f(self.origin),
        }
    }
}

impl<T, Extra> Size for Point<T, Extra> {
    const SIZE: usize = HASH_SIZE;
    type Size = typenum::generic_const_mappings::U<HASH_SIZE>;
}

impl<T: Object> Point<T> {
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self::from_address_extra(address, resolve, ())
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Point<T, Extra> {
    pub fn from_address_extra(address: Address, resolve: Arc<dyn Resolve>, extra: Extra) -> Self {
        Self::from_origin(
            address.hash,
            Arc::new(ByAddress::from_inner(
                ByAddressInner { address, resolve },
                extra,
            )),
        )
    }

    pub fn with_resolve(&self, resolve: Arc<dyn Resolve>) -> Self {
        Self::from_address_extra(
            Address::from_hash(self.hash()),
            resolve,
            self.extra().clone(),
        )
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

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.resolve.resolve_extension(self.address, typeid)
    }
}

struct ByAddress<T, Extra> {
    inner: ByAddressInner,
    extra: Extra,
    _object: PhantomData<fn() -> T>,
}

impl<T, Extra: 'static + Clone> FromInner for ByAddress<T, Extra> {
    type Inner = ByAddressInner;
    type Extra = Extra;

    fn from_inner(inner: Self::Inner, extra: Self::Extra) -> Self {
        Self {
            inner,
            extra,
            _object: PhantomData,
        }
    }
}

impl<T, Extra: 'static> FetchBytes for ByAddress<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.inner.fetch_bytes()
    }

    fn as_inner(&self) -> Option<&dyn Any> {
        Some(&self.inner)
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.inner.extension(typeid)
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync> Fetch for ByAddress<T, Extra> {
    type T = T;
    type Extra = Extra;

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

    fn extra(&self) -> &Self::Extra {
        &self.extra
    }
}

pub trait PointVisitor<Extra: 'static = ()> {
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>);
}

struct HashVisitor<F>(F);

impl<F: FnMut(Hash), Extra: 'static> PointVisitor<Extra> for HashVisitor<F> {
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        self.0(point.hash());
    }
}

pub struct ReflessInput<'a> {
    data: Option<&'a [u8]>,
}

pub struct Input<'a, Extra = ()> {
    refless: ReflessInput<'a>,
    resolve: &'a Arc<dyn Resolve>,
    index: &'a Cell<usize>,
    extra: &'a Extra,
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

impl ParseInput for ReflessInput<'_> {
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

    fn parse_n<'a>(&mut self, n: usize) -> crate::Result<&'a [u8]>
    where
        Self: 'a,
    {
        match self.data()?.split_at_checked(n) {
            Some((chunk, data)) => {
                self.data = Some(data);
                Ok(chunk)
            }
            None => self.end_of_input(),
        }
    }

    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T> {
        let input = ReflessInput {
            data: Some(self.parse_n(n)?),
        };
        T::parse(input)
    }

    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>> {
        match self.data()?.split_at_checked(n) {
            Some((chunk, data)) => {
                if chunk == c {
                    self.data = Some(data);
                    Ok(None)
                } else {
                    self.parse_inline().map(Some)
                }
            }
            None => self.end_of_input(),
        }
    }

    fn parse_all<'a>(self) -> crate::Result<&'a [u8]>
    where
        Self: 'a,
    {
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

impl<Extra> ParseInput for Input<'_, Extra> {
    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a,
    {
        (**self).parse_chunk()
    }

    fn parse_n<'a>(&mut self, n: usize) -> crate::Result<&'a [u8]>
    where
        Self: 'a,
    {
        (**self).parse_n(n)
    }

    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T> {
        let input = Input {
            refless: ReflessInput {
                data: Some(self.parse_n(n)?),
            },
            resolve: self.resolve,
            index: self.index,
            extra: self.extra,
        };
        T::parse(input)
    }

    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>> {
        match self.data()?.split_at_checked(n) {
            Some((chunk, data)) => {
                if chunk == c {
                    self.data = Some(data);
                    Ok(None)
                } else {
                    self.parse_inline().map(Some)
                }
            }
            None => self.end_of_input(),
        }
    }

    fn parse_all<'a>(self) -> crate::Result<&'a [u8]>
    where
        Self: 'a,
    {
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

impl<'a, Extra: 'static + Clone> PointInput for Input<'a, Extra> {
    type Extra = Extra;
    type WithExtra<E: 'static + Clone> = Input<'a, E>;

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

impl<Extra: 'static> PointVisitor<Extra> for CountVisitor {
    fn visit<T: Object<Extra>>(&mut self, _: &Point<T, Extra>) {
        self.count += 1;
    }
}

pub trait Topological<Extra: 'static = ()> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

pub trait FullHash<Extra: 'static>: ToOutput + Topological<Extra> + Tagged {
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

impl<T: ?Sized + ToOutput + Topological<Extra> + Tagged, Extra: 'static> FullHash<Extra> for T {}

pub trait Object<Extra: 'static = ()>:
    'static + Sized + Send + Sync + FullHash<Extra> + for<'a> Parse<Input<'a, Extra>>
{
    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        let _ = typeid;
        Err(Error::UnknownExtension)
    }

    fn point_extra(self, extra: Extra) -> Point<Self, Extra>
    where
        Self: Clone,
        Extra: Send + Sync + Clone,
    {
        Point::from_object_extra(self, extra)
    }

    fn to_resolve(&self) -> Arc<dyn Resolve>
    where
        Self: Clone,
    {
        Arc::new(ByTopology {
            topology: self.topology(),
            extension: Box::new(self.clone()),
        })
    }
}

pub trait SimpleObject: Object {
    fn point(self) -> Point<Self>
    where
        Self: Clone,
    {
        self.point_extra(())
    }
}

impl<T: Object> SimpleObject for T {}

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

pub trait Inline<Extra: 'static = ()>:
    Object<Extra> + for<'a> ParseInline<Input<'a, Extra>>
{
}

impl<T: Object<Extra>, Extra: 'static> Topological<Extra> for Point<T, Extra> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
        visitor.visit(self);
    }

    fn point_count(&self) -> usize {
        1
    }
}

impl<T: Object<I::Extra>, I: PointInput<Extra: Send + Sync>> ParseInline<I> for Point<T, I::Extra> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self::from_address_extra(
            input.parse_inline()?,
            input.resolve(),
            input.extra().clone(),
        ))
    }
}

impl<T: Tagged, Extra> Tagged for Point<T, Extra> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Object<Extra> for Point<T, Extra> {}

impl<T, Extra> ToOutput for Point<T, Extra> {
    fn to_output(&self, output: &mut dyn Output) {
        self.hash().to_output(output);
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Inline<Extra> for Point<T, Extra> {}

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

impl<Extra: 'static> PointVisitor<Extra> for TopoVec {
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        self.push(Arc::new(point.clone()));
    }
}

impl<T, Extra> FetchBytes for Point<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.origin.fetch_bytes()
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.origin.extension(typeid)
    }
}

impl<T, Extra> Singular for Point<T, Extra> {
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

pub trait ReflessInline: ReflessObject + for<'a> ParseInline<ReflessInput<'a>> {
    fn parse_as_inline(mut input: ReflessInput) -> crate::Result<Self> {
        let object = Self::parse_inline(&mut input)?;
        input.empty()?;
        Ok(object)
    }
}

pub trait Output {
    fn write(&mut self, data: &[u8]);
}

impl Output for Vec<u8> {
    fn write(&mut self, data: &[u8]) {
        self.extend_from_slice(data);
    }
}

impl Output for Vec<&'static str> {
    fn write(&mut self, data: &[u8]) {
        let _ = data;
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

pub struct PointMut<'a, T: Object<Extra>, Extra: 'static = ()> {
    hash: &'a mut OptionalHash,
    origin: &'a mut dyn Fetch<T = T, Extra = Extra>,
}

impl<T: Object<Extra>, Extra> Deref for PointMut<'_, T, Extra> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.origin.get().unwrap()
    }
}

impl<T: Object<Extra>, Extra> DerefMut for PointMut<'_, T, Extra> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.origin.get_mut().unwrap()
    }
}

impl<T: Object<Extra>, Extra> Drop for PointMut<'_, T, Extra> {
    fn drop(&mut self) {
        self.finalize();
    }
}

impl<'a, T: Object<Extra>, Extra> PointMut<'a, T, Extra> {
    fn finalize(&mut self) {
        self.origin.get_mut_finalize();
        *self.hash = self.full_hash().into();
    }
}

impl<T: Object + Clone> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self::from_object_extra(object, ())
    }
}

impl<T: Object<Extra> + Clone, Extra: 'static + Send + Sync + Clone> Point<T, Extra> {
    pub fn from_object_extra(object: T, extra: Extra) -> Self {
        Self::from_origin(object.full_hash(), Arc::new(LocalOrigin { object, extra }))
    }

    fn yolo_mut(&mut self) -> bool {
        self.origin.get().is_some()
            && Arc::get_mut(&mut self.origin).is_some_and(|origin| origin.get_mut().is_some())
    }

    async fn prepare_yolo_origin(&mut self) -> crate::Result<()> {
        if !self.yolo_mut() {
            let object = self.origin.fetch().await?;
            self.origin = Arc::new(LocalOrigin {
                object,
                extra: self.origin.extra().clone(),
            });
        }
        Ok(())
    }

    pub async fn fetch_mut(&'_ mut self) -> crate::Result<PointMut<'_, T, Extra>> {
        self.prepare_yolo_origin().await?;
        let origin = Arc::get_mut(&mut self.origin).unwrap();
        assert!(origin.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            origin,
        })
    }

    pub async fn fetch_ref(&mut self) -> crate::Result<&T> {
        self.prepare_yolo_origin().await?;
        Ok(self.origin.get().unwrap())
    }
}

struct LocalOrigin<T, Extra> {
    object: T,
    extra: Extra,
}

impl<T, Extra> Deref for LocalOrigin<T, Extra> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T, Extra> DerefMut for LocalOrigin<T, Extra> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<T: Object<Extra> + Clone, Extra: 'static + Send + Sync> Fetch for LocalOrigin<T, Extra> {
    type T = T;
    type Extra = Extra;

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

    fn extra(&self) -> &Self::Extra {
        &self.extra
    }
}

impl<T: Object<Extra> + Clone, Extra: 'static> FetchBytes for LocalOrigin<T, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(ready(Ok((self.object.output(), self.object.to_resolve()))))
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.object.extension(typeid)
    }
}

trait AsExtension<Extra> {
    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any>;
}

impl<T: Object<Extra>, Extra: 'static> AsExtension<Extra> for T {
    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.extension(typeid)
    }
}

struct ByTopology<Extra> {
    topology: TopoVec,
    extension: Box<dyn Send + Sync + AsExtension<Extra>>,
}

impl<Extra> ByTopology<Extra> {
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
}

impl<Extra> Resolve for ByTopology<Extra> {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        self.try_resolve(address)
            .map_err(Err)
            .map_err(ready)
            .map_err(Box::pin)
            .unwrap_or_else(|x| x)
    }

    fn resolve_extension(&self, address: Address, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.topology
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?
            .extension(typeid)
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.extension.extension(typeid)
    }

    fn name(&self) -> &str {
        "by topology"
    }
}

impl<T: Object<Extra>, Extra: 'static + Send + Sync> Fetch for Point<T, Extra> {
    type T = T;
    type Extra = Extra;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        self.origin.fetch_full()
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        self.origin.fetch()
    }

    fn get(&self) -> Option<&Self::T> {
        self.origin.get()
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        self.hash.clear();
        Arc::get_mut(&mut self.origin)?.get_mut()
    }

    fn get_mut_finalize(&mut self) {
        let origin = Arc::get_mut(&mut self.origin).unwrap();
        origin.get_mut_finalize();
        self.hash = origin.get().unwrap().full_hash().into();
    }

    fn extra(&self) -> &Self::Extra {
        self.origin.extra()
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

    fn iter_accept_points<Extra: 'static>(self, visitor: &mut impl PointVisitor<Extra>)
    where
        Self::Item: Topological<Extra>,
    {
        self.into_iter()
            .for_each(|item| item.accept_points(visitor));
    }
}

pub trait ParseInput: Sized {
    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a;
    fn parse_n<'a>(&mut self, n: usize) -> crate::Result<&'a [u8]>
    where
        Self: 'a;
    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T>;
    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>>;
    fn parse_all<'a>(self) -> crate::Result<&'a [u8]>
    where
        Self: 'a;
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
                Ok(input) => input,
                Err(e) => return Some(Err(e)),
            }?;
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
impl<Extra: 'static> Topological<Extra> for Extras<Extra> {}
impl<Extra: 'static + Send + Sync + Clone> Object<Extra> for Extras<Extra> {}
impl<Extra: 'static + Send + Sync + Clone> Inline<Extra> for Extras<Extra> {}

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
impl<U: 'static + Equivalent<T>, T: 'static, Extra: 'static> Equivalent<Point<T, Extra>>
    for Point<U, Extra>
{
    fn into_equivalent(self) -> Point<T, Extra> {
        self.map_origin(|origin| {
            Arc::new(MapEquivalent {
                origin,
                map: U::into_equivalent,
            })
        })
    }

    fn from_equivalent(point: Point<T, Extra>) -> Self {
        point.map_origin(|origin| {
            Arc::new(MapEquivalent {
                origin,
                map: U::from_equivalent,
            })
        })
    }
}

struct MapEquivalent<T, F, Extra> {
    origin: Arc<dyn Fetch<T = T, Extra = Extra>>,
    map: F,
}

impl<T, F, Extra> FetchBytes for MapEquivalent<T, F, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.origin.fetch_bytes()
    }

    fn extension(&self, typeid: TypeId) -> crate::Result<&dyn Any> {
        self.origin.extension(typeid)
    }
}

trait Map1<T>: Fn(T) -> Self::U {
    type U;
}

impl<T, U, F: Fn(T) -> U> Map1<T> for F {
    type U = U;
}

impl<T, F: 'static + Send + Sync + Map1<T>, Extra> Fetch for MapEquivalent<T, F, Extra> {
    type T = F::U;
    type Extra = Extra;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
    }

    fn extra(&self) -> &Self::Extra {
        self.origin.extra()
    }
}

impl<T: 'static + ToOutput, Extra: 'static> Point<T, Extra> {
    pub fn map<U>(self, map: impl 'static + Send + Sync + Fn(T) -> U) -> Point<U, Extra> {
        self.map_origin(|origin| Arc::new(Map { origin, map }))
    }
}

struct Map<T, F, Extra> {
    origin: Arc<dyn Fetch<T = T, Extra = Extra>>,
    map: F,
}

impl<T: ToOutput, F, Extra> FetchBytes for Map<T, F, Extra> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| (x.output(), r)))
    }
}

impl<T: ToOutput, F: 'static + Send + Sync + Map1<T>, Extra> Fetch for Map<T, F, Extra> {
    type T = F::U;
    type Extra = Extra;

    fn fetch_full(&'_ self) -> FailFuture<'_, (Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
    }

    fn extra(&self) -> &Self::Extra {
        self.origin.extra()
    }
}

impl<T, Extra> MaybeHasNiche for Point<T, Extra> {
    type MnArray = <Hash as MaybeHasNiche>::MnArray;
}

impl<T: FullHash<Extra> + Default, Extra: 'static> Point<T, Extra> {
    pub fn is_default(&self) -> bool {
        self.hash() == T::default().full_hash()
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
