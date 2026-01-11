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
use generic_array::{ArrayLength, GenericArray};
pub use object_rainbow_derive::{
    Enum, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseAsInline, ParseInline, Size, Tagged,
    ToOutput, Topological,
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
mod sha2_const;
pub mod zero_terminated;

/// SHA-256 hash size in bytes.
pub const HASH_SIZE: usize = sha2_const::Sha256::DIGEST_SIZE;

/// Address within a [`PointInput`].
///
/// This was introduced:
/// - to avoid using a [`Hash`]-only map
/// - to differentiate between separate [`Hash`]es within a context
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

pub type Node<T> = (T, Arc<dyn Resolve>);

/// Returned by [`Resolve`] and [`FetchBytes`]. Represents traversal through the object graph.
pub type ByteNode = Node<Vec<u8>>;

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
    fn try_resolve_local(&self, address: Address) -> Result<Option<ByteNode>> {
        let _ = address;
        Ok(None)
    }
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
    fn fetch_bytes_local(&self) -> Result<Option<ByteNode>> {
        Ok(None)
    }
    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        None
    }
    fn as_inner(&self) -> Option<&dyn Any> {
        None
    }
}

pub trait Fetch: Send + Sync + FetchBytes {
    type T;
    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>>;
    fn fetch(&'_ self) -> FailFuture<'_, Self::T>;
    fn try_fetch_local(&self) -> Result<Option<Node<Self::T>>> {
        Ok(None)
    }
    fn fetch_local(&self) -> Option<Self::T> {
        None
    }
    fn get(&self) -> Option<&Self::T> {
        None
    }
    fn get_mut(&mut self) -> Option<&mut Self::T> {
        None
    }
    fn get_mut_finalize(&mut self) {}
}

#[derive(ToOutput, InlineOutput, ListHashes, Topological, Parse, ParseInline)]
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

pub trait PointVisitor {
    fn visit<T: Traversible>(&mut self, point: &(impl 'static + SingularFetch<T = T> + Clone));
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

    fn output<T: Output + Default>(&self) -> T {
        let mut output = T::default();
        self.to_output(&mut output);
        output
    }

    fn vec(&self) -> Vec<u8> {
        self.output()
    }
}

/// Marker trait indicating that [`ToOutput`] result cannot be extended.
pub trait InlineOutput: ToOutput {
    fn slice_to_output(slice: &[Self], output: &mut dyn Output)
    where
        Self: Sized,
    {
        slice.iter_to_output(output);
    }
}

pub trait ListHashes {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        let _ = f;
    }

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        self.list_hashes(&mut |hash| hasher.update(hash));
        Hash::from_sha256(hasher.finalize().into())
    }

    fn point_count(&self) -> usize {
        let mut count = 0;
        self.list_hashes(&mut |_| count += 1);
        count
    }
}

pub trait Topological: ListHashes {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        let _ = visitor;
    }

    fn topology(&self) -> TopoVec {
        let mut topology = TopoVec::with_capacity(self.point_count());
        self.traverse(&mut topology);
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

    fn reparse(&self) -> crate::Result<Self>
    where
        Self: Traversible,
    {
        self.reparse_extra(&())
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

    fn reparse_extra(&self, extra: &Extra) -> crate::Result<Self>
    where
        Self: Traversible,
    {
        Self::parse_slice_extra(&self.vec(), &self.to_resolve(), extra)
    }
}

impl<T: for<'a> Parse<Input<'a, Extra>>, Extra> ParseSliceExtra<Extra> for T {}

#[derive(ToOutput)]
pub struct ObjectHashes {
    pub tags: Hash,
    pub topology: Hash,
    pub data: Hash,
}

pub trait FullHash: ToOutput + ListHashes + Tagged {
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

impl<T: ?Sized + ToOutput + ListHashes + Tagged> FullHash for T {}

pub trait Traversible: 'static + Sized + Send + Sync + FullHash + Topological {
    fn to_resolve(&self) -> Arc<dyn Resolve> {
        Arc::new(ByTopology {
            topology: self.topology(),
        })
    }
}

impl<T: 'static + Send + Sync + FullHash + Topological> Traversible for T {}

pub trait Object<Extra = ()>: Traversible + for<'a> Parse<Input<'a, Extra>> {}

impl<T: Traversible + for<'a> Parse<Input<'a, Extra>>, Extra> Object<Extra> for T {}

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

pub trait Inline<Extra = ()>:
    Object<Extra> + InlineOutput + for<'a> ParseInline<Input<'a, Extra>>
{
}

impl<T: Object<Extra> + InlineOutput + for<'a> ParseInline<Input<'a, Extra>>, Extra> Inline<Extra>
    for T
{
}

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

pub trait SingularFetch: Singular + Fetch {}

impl<T: ?Sized + Singular + Fetch> SingularFetch for T {}

pub type TopoVec = Vec<Arc<dyn Singular>>;

impl PointVisitor for TopoVec {
    fn visit<T: Traversible>(&mut self, point: &(impl 'static + SingularFetch<T = T> + Clone)) {
        self.push(Arc::new(point.clone()));
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

impl<T: 'static + Sized + Send + Sync + ToOutput + Tagged + for<'a> Parse<ReflessInput<'a>>>
    ReflessObject for T
{
}

pub trait ReflessInline:
    ReflessObject + InlineOutput + for<'a> ParseInline<ReflessInput<'a>>
{
}

impl<T: ReflessObject + InlineOutput + for<'a> ParseInline<ReflessInput<'a>>> ReflessInline for T {}

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

    fn try_resolve_local(&self, address: Address) -> Result<Option<ByteNode>> {
        let point = self
            .topology
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        if point.hash() != address.hash {
            Err(Error::ResolutionMismatch)
        } else {
            point.fetch_bytes_local()
        }
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
        Self::Item: InlineOutput,
    {
        self.into_iter().for_each(|item| item.to_output(output));
    }

    fn iter_list_hashes(self, f: &mut impl FnMut(Hash))
    where
        Self::Item: ListHashes,
    {
        self.into_iter().for_each(|item| item.list_hashes(f));
    }

    fn iter_traverse(self, visitor: &mut impl PointVisitor)
    where
        Self::Item: Topological,
    {
        self.into_iter().for_each(|item| item.traverse(visitor));
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

pub trait ExtraFor<T> {
    fn parse(&self, data: &[u8], resolve: &Arc<dyn Resolve>) -> Result<T>;

    fn parse_checked(&self, hash: Hash, data: &[u8], resolve: &Arc<dyn Resolve>) -> Result<T>
    where
        T: FullHash,
    {
        let object = self.parse(data, resolve)?;
        if object.full_hash() != hash {
            Err(Error::FullHashMismatch)
        } else {
            Ok(object)
        }
    }
}

impl<T: for<'a> Parse<Input<'a, Extra>>, Extra> ExtraFor<T> for Extra {
    fn parse(&self, data: &[u8], resolve: &Arc<dyn Resolve>) -> Result<T> {
        T::parse_slice_extra(data, resolve, self)
    }
}

pub trait BoundPair: Sized {
    type T;
    type E;
}

impl<T, E> BoundPair for (T, E) {
    type T = T;
    type E = E;
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
    assert_eq!(Option::<Hash>::SIZE, HASH_SIZE);
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
    assert_eq!(Option::<Hash>::SIZE, HASH_SIZE);
    assert_eq!(Option::<Option<Hash>>::SIZE, HASH_SIZE);
    assert_eq!(Option::<Option<Option<Hash>>>::SIZE, HASH_SIZE);
}
