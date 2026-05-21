#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

extern crate self as object_rainbow;

use std::{
    any::Any,
    borrow::Cow,
    cell::Cell,
    cmp::Ordering,
    convert::Infallible,
    future::ready,
    marker::PhantomData,
    ops::{Add, Deref, DerefMut, Sub},
    pin::Pin,
    sync::Arc,
};

pub use anyhow::anyhow;
use generic_array::{ArrayLength, GenericArray, functional::FunctionalSequence, sequence::Split};
pub use object_rainbow_derive::{
    Enum, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseAsInline, ParseInline, Size, Tagged,
    ToOutput, Topological, derive_for_wrapped,
};
use sha2::{Digest, Sha256};
#[doc(hidden)]
pub use typenum;
use typenum::Unsigned;

#[doc(hidden)]
pub use self::niche::{MaybeNiche, MnArray, NicheFoldOrArray, NicheOr};
pub use self::{
    enumkind::Enum,
    error::{Error, Result},
    hash::{Hash, OptionalHash},
    niche::{
        AutoEnumNiche, AutoNiche, HackNiche, MaybeHasNiche, Niche, NicheForUnsized, NoNiche,
        OneNiche, SomeNiche, ZeroNiche, ZeroNoNiche,
    },
    ordering::{ByteOrd, OrderedByBytes, SignificantLength},
};

mod assert_impl;
pub mod enumkind;
mod error;
mod hash;
pub mod hashed;
mod impls;
pub mod incr_byte_niche;
pub mod inline_extra;
pub mod length_prefixed;
pub mod map_extra;
mod niche;
pub mod numeric;
pub mod object_marker;
mod ordering;
pub mod parse_extra;
pub mod partial_byte_tag;
pub mod tuple_extra;
pub mod u63;
pub mod with_repr;
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

impl ToOutput for Address {
    fn to_output(&self, output: &mut impl Output) {
        self.hash.to_output(output);
    }
}

impl InlineOutput for Address {}

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

/// Something that can resolve [`Address`]es to [`ByteNode`]s.
pub trait Resolve: Send + Sync + AsAny {
    /// Resolve the address. For an [`Object`], this is what gets used as [`PointInput`].
    fn resolve<'a>(
        &'a self,
        address: Address,
        this: &'a Arc<dyn Resolve>,
    ) -> FailFuture<'a, ByteNode>;
    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>>;
    fn try_resolve_local(
        &self,
        address: Address,
        this: &Arc<dyn Resolve>,
    ) -> Result<Option<ByteNode>> {
        let _ = address;
        let _ = this;
        Ok(None)
    }
    fn topology_hash(&self) -> Option<Hash> {
        None
    }
    fn into_topovec(self: Arc<Self>) -> Option<TopoVec> {
        None
    }
}

impl ToOutput for dyn Resolve {
    fn to_output(&self, _: &mut impl Output) {}
}

impl InlineOutput for dyn Resolve {}

impl<I: PointInput> Parse<I> for Arc<dyn Resolve> {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: PointInput> ParseInline<I> for Arc<dyn Resolve> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(input.resolve())
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
    fn as_resolve(&self) -> Option<&Arc<dyn Resolve>> {
        None
    }
    fn try_unwrap_resolve(self: Arc<Self>) -> Option<Arc<dyn Resolve>> {
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
    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        None
    }
    fn into_dyn_fetch<'a>(self) -> Arc<dyn 'a + Fetch<T = Self::T>>
    where
        Self: 'a + Sized,
    {
        Arc::new(self)
    }
}

pub trait PointVisitor {
    fn visit(&mut self, point: &(impl 'static + SingularFetch<T: Traversible> + Clone));
}

struct ReflessData<'d> {
    slice: &'d [u8],
    prefix: Vec<Vec<u8>>,
}

impl<'a> ReflessData<'a> {
    fn split_at_checked(&self, mut n: usize) -> Option<(Self, Self)> {
        let mut prefix_l = Vec::new();
        let mut prefix_r = self.prefix.clone();
        while n > 0
            && let Some(front) = prefix_r.last_mut()
        {
            if n < front.len() {
                n = 0;
                prefix_l.push(Vec::from(&front[..n]));
                front.drain(..n);
            } else {
                n -= front.len();
                prefix_l.push(prefix_r.pop().expect("last element is known to exist"));
            }
        }
        prefix_l.reverse();
        self.slice.split_at_checked(n).map(|(slice_l, slice_r)| {
            (
                Self {
                    slice: slice_l,
                    prefix: prefix_l,
                },
                Self {
                    slice: slice_r,
                    prefix: prefix_r,
                },
            )
        })
    }

    fn len(&self) -> usize {
        self.slice.len() + self.prefix.iter().map(|v| v.len()).sum::<usize>()
    }

    fn is_empty(&self) -> bool {
        self.slice.is_empty() && self.prefix.iter().all(|v| v.is_empty())
    }

    fn starts_with(&self, mut prefix: &[u8]) -> bool {
        for chunk in self.prefix.iter().rev() {
            if chunk.starts_with(prefix) {
                return true;
            }
            if let Some(rest) = prefix.strip_prefix(&**chunk) {
                prefix = rest;
            } else {
                return false;
            }
        }
        self.slice.starts_with(prefix)
    }

    fn cow(&self) -> Cow<'a, [u8]> {
        if self.prefix.iter().all(|v| v.is_empty()) {
            Cow::from(self.slice)
        } else {
            let mut vec = Vec::with_capacity(self.len());
            for v in self.prefix.iter().rev() {
                vec.extend_from_slice(v);
            }
            vec.extend_from_slice(self.slice);
            Cow::from(vec)
        }
    }

    fn iter(&self) -> impl Iterator<Item = &u8> {
        self.prefix.iter().rev().flatten().chain(self.slice)
    }
}

pub struct ReflessInput<'d> {
    data: Option<ReflessData<'d>>,
}

pub struct Input<'d, Extra: Clone = ()> {
    refless: ReflessInput<'d>,
    resolve: Cow<'d, Arc<dyn Resolve>>,
    index: &'d Cell<usize>,
    extra: Cow<'d, Extra>,
}

impl<'a, Extra: Clone> Deref for Input<'a, Extra> {
    type Target = ReflessInput<'a>;

    fn deref(&self) -> &Self::Target {
        &self.refless
    }
}

impl<Extra: Clone> DerefMut for Input<'_, Extra> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.refless
    }
}

impl<'a> ReflessInput<'a> {
    fn data(&self) -> crate::Result<&ReflessData<'a>> {
        self.data.as_ref().ok_or(Error::EndOfInput)
    }

    fn data_mut(&mut self) -> crate::Result<&mut ReflessData<'a>> {
        self.data.as_mut().ok_or(Error::EndOfInput)
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
    type Data = Cow<'d, [u8]>;

    fn push_front(&mut self, data: impl Into<Vec<u8>>) -> crate::Result<()> {
        let data = data.into();
        if !data.is_empty() {
            self.data_mut()?.prefix.push(data);
        }
        Ok(())
    }

    fn read(&mut self, mut data: &mut [u8]) -> crate::Result<()> {
        match self.data()?.split_at_checked(data.len()) {
            Some((chunk, rest)) => {
                self.data = Some(rest);
                for v in chunk.prefix.iter().rev() {
                    let part;
                    (part, data) = data.split_at_mut(v.len());
                    part.copy_from_slice(v);
                }
                data.copy_from_slice(chunk.slice);
                Ok(())
            }
            None => self.end_of_input(),
        }
    }

    fn split_n(&mut self, n: usize) -> crate::Result<Self> {
        match self.data()?.split_at_checked(n) {
            Some((chunk, rest)) => {
                self.data = Some(rest);
                Ok(Self { data: Some(chunk) })
            }
            None => self.end_of_input(),
        }
    }

    fn skip_n(&mut self, n: usize) -> crate::Result<()> {
        match self.data()?.split_at_checked(n) {
            Some((_, rest)) => {
                self.data = Some(rest);
                Ok(())
            }
            None => self.end_of_input(),
        }
    }

    fn find_zero(&mut self) -> crate::Result<usize> {
        let found = self.data()?.iter().enumerate().find(|(_, x)| **x == 0);
        match found {
            Some((at, _)) => Ok(at),
            None => self.end_of_input(),
        }
    }

    fn parse_n_ahead(&mut self, n: usize) -> crate::Result<Vec<u8>> {
        match self.data()?.split_at_checked(n) {
            Some((data, _)) => Ok(data.cow().into_owned()),
            None => self.end_of_input(),
        }
    }

    fn compare_ahead(&mut self, c: &[u8]) -> crate::Result<bool> {
        let data = self.data()?;
        if data.len() < c.len() {
            self.end_of_input()
        } else {
            Ok(data.starts_with(c))
        }
    }

    fn parse_all(self) -> crate::Result<Self::Data> {
        self.data().map(|data| data.cow())
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

    fn parse_refless_inline<T: for<'r> ParseInline<ReflessInput<'r>>>(
        &mut self,
    ) -> crate::Result<T> {
        self.parse_inline()
    }

    fn parse_refless<T: for<'r> Parse<ReflessInput<'r>>>(self) -> crate::Result<T> {
        self.parse()
    }
}

impl<'d, Extra: Clone> ParseInput for Input<'d, Extra> {
    type Data = Cow<'d, [u8]>;

    fn push_front(&mut self, data: impl Into<Vec<u8>>) -> crate::Result<()> {
        (**self).push_front(data)
    }

    fn read(&mut self, data: &mut [u8]) -> crate::Result<()> {
        (**self).read(data)
    }

    fn split_n(&mut self, n: usize) -> crate::Result<Self> {
        Ok(Self {
            refless: self.refless.split_n(n)?,
            resolve: self.resolve.clone(),
            index: self.index,
            extra: self.extra.clone(),
        })
    }

    fn skip_n(&mut self, n: usize) -> crate::Result<()> {
        (**self).skip_n(n)
    }

    fn find_zero(&mut self) -> crate::Result<usize> {
        (**self).find_zero()
    }

    fn parse_n_ahead(&mut self, n: usize) -> crate::Result<Vec<u8>> {
        (**self).parse_n_ahead(n)
    }

    fn compare_ahead(&mut self, c: &[u8]) -> crate::Result<bool> {
        (**self).compare_ahead(c)
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

    fn parse_refless_inline<T: for<'r> ParseInline<ReflessInput<'r>>>(
        &mut self,
    ) -> crate::Result<T> {
        (**self).parse_refless_inline()
    }

    fn parse_refless<T: for<'r> Parse<ReflessInput<'r>>>(self) -> crate::Result<T> {
        self.refless.parse_refless()
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
        &self.resolve
    }

    fn with_resolve(mut self, resolve: Arc<dyn Resolve>) -> Self {
        self.resolve = Cow::Owned(resolve);
        self
    }

    fn extra(&self) -> &Self::Extra {
        &self.extra
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
            extra: match extra {
                Cow::Borrowed(extra) => Cow::Borrowed(f(extra)),
                Cow::Owned(extra) => Cow::Owned(f(&extra).clone()),
            },
        }
    }

    fn replace_extra<E: 'static + Clone>(self, e: E) -> (Extra, Self::WithExtra<E>) {
        let Self {
            refless,
            resolve,
            index,
            extra,
        } = self;
        (
            extra.into_owned(),
            Input {
                refless,
                resolve,
                index,
                extra: Cow::Owned(e),
            },
        )
    }

    fn with_extra<E: 'static + Clone>(self, extra: E) -> Self::WithExtra<E> {
        let Self {
            refless,
            resolve,
            index,
            ..
        } = self;
        Input {
            refless,
            resolve,
            index,
            extra: Cow::Owned(extra),
        }
    }

    fn parse_inline_extra<E: 'static + Clone, T: ParseInline<Self::WithExtra<E>>>(
        &mut self,
        extra: E,
    ) -> crate::Result<T> {
        let Self {
            refless,
            resolve,
            index,
            ..
        } = self;
        let data = refless.data.take();
        let resolve = resolve.clone();
        let mut input = Input {
            refless: ReflessInput { data },
            resolve,
            index,
            extra: Cow::Owned(extra),
        };
        let value = input.parse_inline()?;
        refless.data = input.refless.data.take();
        Ok(value)
    }
}

/// Values of this type can be uniquely represented as a `Vec<u8>`.
pub trait ToOutput {
    fn to_output(&self, output: &mut impl Output);

    fn data_hash(&self) -> Hash {
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

        let mut output = HashOutput::default();
        self.to_output(&mut output);
        output.hash()
    }

    fn mangle_hash(&self) -> Hash {
        Mangled(self).data_hash()
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

/// Marker trait indicating that [`ToOutput`] result cannot be extended (no value, when represented
/// as a `Vec<u8>`, may be a prefix of another value).
pub trait InlineOutput: ToOutput {
    fn slice_to_output(slice: &[Self], output: &mut impl Output)
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
    const HASH: Hash = Self::TAGS.hash();
}

pub trait ParseSlice: for<'a> Parse<Input<'a>> {
    fn parse_slice(slice: &[u8], resolve: &Arc<dyn Resolve>) -> crate::Result<Self> {
        Self::parse_slice_extra(slice, resolve, &())
    }

    fn reparse(&self) -> crate::Result<Self>
    where
        Self: Traversible,
    {
        self.reparse_extra(&())
    }
}

impl<T: for<'a> Parse<Input<'a>>> ParseSlice for T {}

pub trait ParseSliceExtra<Extra: Clone>: for<'a> Parse<Input<'a, Extra>> {
    fn parse_slice_extra(
        slice: &[u8],
        resolve: &Arc<dyn Resolve>,
        extra: &Extra,
    ) -> crate::Result<Self> {
        let input = Input {
            refless: ReflessInput {
                data: Some(ReflessData {
                    slice,
                    prefix: Vec::new(),
                }),
            },
            resolve: Cow::Borrowed(resolve),
            index: &Cell::new(0),
            extra: Cow::Borrowed(extra),
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

impl<T: for<'a> Parse<Input<'a, Extra>>, Extra: Clone> ParseSliceExtra<Extra> for T {}

#[derive(ToOutput)]
pub struct DiffHashes {
    pub tags: Hash,
    pub topology: Hash,
    pub mangle: Hash,
}

#[derive(ToOutput)]
pub struct ObjectHashes {
    pub diff: Hash,
    pub data: Hash,
}

pub trait FullHash: ToOutput + ListHashes + Tagged {
    fn diff_hashes(&self) -> DiffHashes {
        DiffHashes {
            tags: Self::HASH,
            topology: self.topology_hash(),
            mangle: self.mangle_hash(),
        }
    }

    fn hashes(&self) -> ObjectHashes {
        ObjectHashes {
            diff: self.diff_hashes().data_hash(),
            data: self.data_hash(),
        }
    }

    fn full_hash(&self) -> Hash {
        self.hashes().data_hash()
    }
}

impl<T: ?Sized + ToOutput + ListHashes + Tagged> FullHash for T {}

pub trait DefaultHash: FullHash + Default {
    fn default_hash() -> Hash {
        Self::default().full_hash()
    }
}

impl<T: FullHash + Default> DefaultHash for T {}

pub trait Traversible: 'static + Sized + Send + Sync + FullHash + Topological {
    fn to_resolve(&self) -> Arc<dyn Resolve> {
        struct ByTopology {
            topology: TopoVec,
            topology_hash: Hash,
        }

        impl Drop for ByTopology {
            fn drop(&mut self) {
                while let Some(singular) = self.topology.pop() {
                    if let Some(resolve) = singular.try_unwrap_resolve()
                        && let Some(topology) = &mut resolve.into_topovec()
                    {
                        self.topology.append(topology);
                    }
                }
            }
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
            fn resolve(
                &'_ self,
                address: Address,
                _: &Arc<dyn Resolve>,
            ) -> FailFuture<'_, ByteNode> {
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

            fn try_resolve_local(
                &self,
                address: Address,
                _: &Arc<dyn Resolve>,
            ) -> Result<Option<ByteNode>> {
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

            fn topology_hash(&self) -> Option<Hash> {
                Some(self.topology_hash)
            }

            fn into_topovec(self: Arc<Self>) -> Option<TopoVec> {
                Arc::try_unwrap(self)
                    .ok()
                    .as_mut()
                    .map(|Self { topology, .. }| std::mem::take(topology))
            }
        }

        let topology = self.topology();
        let topology_hash = topology.data_hash();
        for singular in &topology {
            if let Some(resolve) = singular.as_resolve()
                && resolve.topology_hash() == Some(topology_hash)
            {
                return resolve.clone();
            }
        }
        Arc::new(ByTopology {
            topology,
            topology_hash,
        })
    }
}

impl<T: 'static + Send + Sync + FullHash + Topological> Traversible for T {}

pub trait Object<Extra = ()>: Traversible + for<'a> Parse<Input<'a, Extra>> {}

impl<T: Traversible + for<'a> Parse<Input<'a, Extra>>, Extra> Object<Extra> for T {}

pub struct Tags(pub &'static [&'static str], pub &'static [&'static Self]);

const fn bytes_compare(l: &[u8], r: &[u8]) -> std::cmp::Ordering {
    let mut i = 0;
    while i < l.len() && i < r.len() {
        if l[i] > r[i] {
            return std::cmp::Ordering::Greater;
        } else if l[i] < r[i] {
            return std::cmp::Ordering::Less;
        } else {
            i += 1;
        }
    }
    if l.len() > r.len() {
        std::cmp::Ordering::Greater
    } else if l.len() < r.len() {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Equal
    }
}

const fn str_compare(l: &str, r: &str) -> std::cmp::Ordering {
    bytes_compare(l.as_bytes(), r.as_bytes())
}

impl Tags {
    const fn min_out(&self, strict_min: Option<&str>, min: &mut Option<&'static str>) {
        {
            let mut i = 0;
            while i < self.0.len() {
                let candidate = self.0[i];
                i += 1;
                if let Some(strict_min) = strict_min
                    && str_compare(candidate, strict_min).is_le()
                {
                    continue;
                }
                if let Some(min) = min
                    && str_compare(candidate, min).is_ge()
                {
                    continue;
                }
                *min = Some(candidate);
            }
        }
        {
            let mut i = 0;
            while i < self.1.len() {
                self.1[i].min_out(strict_min, min);
                i += 1;
            }
        }
        if let Some(l) = min
            && let Some(r) = strict_min
        {
            assert!(str_compare(l, r).is_gt());
        }
    }

    const fn min(&self, strict_min: Option<&str>) -> Option<&'static str> {
        let mut min = None;
        self.min_out(strict_min, &mut min);
        min
    }

    const fn const_hash(&self, mut hasher: sha2_const::Sha256) -> sha2_const::Sha256 {
        let mut last = None;
        let mut i = 0;
        while let Some(next) = self.min(last) {
            i += 1;
            if i > 1000 {
                panic!("{}", next);
            }
            hasher = hasher.update(next.as_bytes());
            last = Some(next);
        }
        hasher
    }

    const fn hash(&self) -> Hash {
        Hash::from_sha256(self.const_hash(sha2_const::Sha256::new()).finalize())
    }
}

#[test]
fn min_out_respects_bounds() {
    let mut min = None;
    Tags(&["c", "b", "a"], &[]).min_out(Some("a"), &mut min);
    assert_eq!(min, Some("b"));
}

#[test]
fn const_hash() {
    assert_ne!(Tags(&["a", "b"], &[]).hash(), Tags(&["a"], &[]).hash());
    assert_eq!(
        Tags(&["a", "b"], &[]).hash(),
        Tags(&["a"], &[&Tags(&["b"], &[])]).hash(),
    );
    assert_eq!(Tags(&["a", "b"], &[]).hash(), Tags(&["b", "a"], &[]).hash());
    assert_eq!(Tags(&["a", "a"], &[]).hash(), Tags(&["a"], &[]).hash());
}

pub trait Inline<Extra = ()>:
    Object<Extra> + InlineOutput + for<'a> ParseInline<Input<'a, Extra>>
{
}

impl<T: Object<Extra> + InlineOutput + for<'a> ParseInline<Input<'a, Extra>>, Extra> Inline<Extra>
    for T
{
}

pub trait Topology: Resolve {
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

impl ToOutput for dyn Singular {
    fn to_output(&self, output: &mut impl Output) {
        self.hash().to_output(output);
    }
}

impl InlineOutput for dyn Singular {}

impl ListHashes for Arc<dyn Singular> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        f(self.hash());
    }

    fn point_count(&self) -> usize {
        1
    }
}

pub type TopoVec = Vec<Arc<dyn Singular>>;

impl PointVisitor for TopoVec {
    fn visit(&mut self, point: &(impl 'static + SingularFetch<T: Traversible> + Clone)) {
        self.push(Arc::new(point.clone()));
    }
}

impl Resolve for TopoVec {
    fn resolve<'a>(
        &'a self,
        address: Address,
        _: &'a Arc<dyn Resolve>,
    ) -> FailFuture<'a, ByteNode> {
        Box::pin(async move {
            let singular = self.get(address.index).ok_or(Error::AddressOutOfBounds)?;
            if singular.hash() != address.hash {
                Err(Error::FullHashMismatch)
            } else {
                singular.fetch_bytes().await
            }
        })
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let singular = self.get(address.index).ok_or(Error::AddressOutOfBounds)?;
            if singular.hash() != address.hash {
                Err(Error::FullHashMismatch)
            } else {
                singular.fetch_data().await
            }
        })
    }

    fn try_resolve_local(
        &self,
        address: Address,
        _: &Arc<dyn Resolve>,
    ) -> Result<Option<ByteNode>> {
        let singular = self.get(address.index).ok_or(Error::AddressOutOfBounds)?;
        if singular.hash() != address.hash {
            Err(Error::FullHashMismatch)
        } else {
            singular.fetch_bytes_local()
        }
    }

    fn topology_hash(&self) -> Option<Hash> {
        Some(self.data_hash())
    }

    fn into_topovec(self: Arc<Self>) -> Option<TopoVec> {
        Some((*self).clone())
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
    fn parse_slice_refless(slice: &[u8]) -> crate::Result<Self> {
        let input = ReflessInput {
            data: Some(ReflessData {
                slice,
                prefix: Vec::new(),
            }),
        };
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
    fn is_mangling(&self) -> bool {
        false
    }
    fn is_real(&self) -> bool {
        !self.is_mangling()
    }
    fn as_write(&mut self) -> AsWrite<'_, Self> {
        AsWrite { output: self }
    }
}

pub struct AsWrite<'a, O: ?Sized> {
    output: &'a mut O,
}

impl<O: ?Sized + Output> std::io::Write for AsWrite<'_, O> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.output.write(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Output for Vec<u8> {
    fn write(&mut self, data: &[u8]) {
        self.extend_from_slice(data);
    }
}

struct MangleOutput<'a, T: ?Sized>(&'a mut T);

impl<'a, T: Output> MangleOutput<'a, T> {
    fn new(output: &'a mut T) -> Self {
        assert!(output.is_real());
        assert!(!output.is_mangling());
        Self(output)
    }
}

impl<T: ?Sized + Output> Output for MangleOutput<'_, T> {
    fn write(&mut self, data: &[u8]) {
        self.0.write(data);
    }

    fn is_mangling(&self) -> bool {
        true
    }
}

pub struct Mangled<T: ?Sized>(T);

impl<T: ?Sized + ToOutput> ToOutput for Mangled<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.0.to_output(&mut MangleOutput::new(output));
    }
}

pub trait Size {
    const SIZE: usize = <Self::Size as Unsigned>::USIZE;
    type Size: Unsigned;
}

pub trait SizeExt: Size<Size: ArrayLength> + ToOutput {
    fn to_array(&self) -> GenericArray<u8, Self::Size> {
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

        let mut array = GenericArray::default();
        let mut output = ArrayOutput {
            data: &mut array,
            offset: 0,
        };
        self.to_output(&mut output);
        output.finalize();
        array
    }

    fn reinterpret<T: FromSized<Size = Self::Size>>(&self) -> T {
        T::from_sized(&self.to_array())
    }
}

impl<T: Size<Size: ArrayLength> + ToOutput> SizeExt for T {}

pub trait FromSized: Size<Size: ArrayLength> {
    fn from_sized(data: &GenericArray<u8, Self::Size>) -> Self;
}

impl<
    A: FromSized<Size = An>,
    B: FromSized<Size = Bn>,
    An,
    Bn: Add<An, Output: ArrayLength + Sub<An, Output = Bn>>,
> FromSized for (A, B)
{
    fn from_sized(data: &GenericArray<u8, Self::Size>) -> Self {
        let (a, b) = data.split();
        (A::from_sized(a), B::from_sized(b))
    }
}

pub trait RainbowIterator: Sized + IntoIterator {
    fn iter_to_output(self, output: &mut impl Output)
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

    fn iter_bytes_cmp(self, other: impl IntoIterator<Item = Self::Item>) -> Ordering
    where
        Self::Item: ByteOrd,
    {
        self.into_iter()
            .map(OrderedByBytes)
            .cmp(other.into_iter().map(OrderedByBytes))
    }
}

pub trait ParseInput: Sized {
    type Data: AsRef<[u8]> + Deref<Target = [u8]> + Into<Vec<u8>>;
    fn push_front(&mut self, data: impl Into<Vec<u8>>) -> crate::Result<()>;
    fn read(&mut self, data: &mut [u8]) -> crate::Result<()>;
    fn parse_chunk<const N: usize>(&mut self) -> crate::Result<[u8; N]> {
        let mut chunk = [0; _];
        self.read(&mut chunk)?;
        Ok(chunk)
    }
    fn split_n(&mut self, n: usize) -> crate::Result<Self>;
    fn skip_n(&mut self, n: usize) -> crate::Result<()>;
    fn find_zero(&mut self) -> crate::Result<usize>;
    fn parse_n_ahead(&mut self, n: usize) -> crate::Result<Vec<u8>>;
    fn compare_ahead(&mut self, c: &[u8]) -> crate::Result<bool>;
    fn split_parse<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T> {
        self.split_n(n)?.parse()
    }
    fn parse_zero_terminated<T: Parse<Self>>(&mut self) -> crate::Result<(Vec<u8>, T)> {
        let n = self.find_zero()?;
        let data = self.parse_n_ahead(n)?;
        let value = self.split_parse(n)?;
        self.skip_n(1)?;
        Ok((data, value))
    }
    fn parse_compare<T: Parse<Self>>(&mut self, c: &[u8]) -> Result<Option<T>> {
        if self.compare_ahead(c)? {
            self.skip_n(c.len())?;
            Ok(None)
        } else {
            Ok(Some(self.split_parse(c.len())?))
        }
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

    fn parse_vec_n<T: ParseInline<Self>>(&mut self, n: usize) -> crate::Result<Vec<T>> {
        T::parse_vec_n(self, n)
    }

    fn parse_array<T: ParseInline<Self>, const N: usize>(&mut self) -> crate::Result<[T; N]> {
        T::parse_array(self)
    }

    fn parse_generic_array<T: ParseInline<Self>, N: ArrayLength>(
        &mut self,
    ) -> crate::Result<GenericArray<T, N>> {
        T::parse_generic_array(self)
    }

    fn as_read<T, E>(
        &mut self,
        f: impl FnOnce(AsRead<'_, Self>) -> std::result::Result<T, E>,
    ) -> crate::Result<T>
    where
        Error: From<E>,
    {
        let result = f(AsRead { input: self })?;
        self.noop()?;
        Ok(result)
    }

    fn noop(&mut self) -> crate::Result<()> {
        self.read(&mut [])
    }

    fn parse_refless_inline<T: for<'r> ParseInline<ReflessInput<'r>>>(
        &mut self,
    ) -> crate::Result<T>;

    fn parse_refless<T: for<'r> Parse<ReflessInput<'r>>>(self) -> crate::Result<T>;
}

pub struct AsRead<'a, I> {
    input: &'a mut I,
}

impl<I: ParseInput> std::io::Read for AsRead<'_, I> {
    fn read(&mut self, data: &mut [u8]) -> std::io::Result<usize> {
        self.read_exact(data)?;
        Ok(data.len())
    }

    fn read_exact(&mut self, data: &mut [u8]) -> std::io::Result<()> {
        self.input.read(data)?;
        Ok(())
    }

    fn read_to_end(&mut self, _: &mut Vec<u8>) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::Unsupported.into())
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
    fn with_resolve(self, resolve: Arc<dyn Resolve>) -> Self;
    /// Get [`Self::Extra`].
    fn extra(&self) -> &Self::Extra;
    /// Project the `Extra`. Under some circumstances, prevents an extra [`Clone::clone`].
    fn map_extra<E: 'static + Clone>(
        self,
        f: impl FnOnce(&Self::Extra) -> &E,
    ) -> Self::WithExtra<E>;
    /// Return the old [`Self::Extra`], give a new [`PointInput`] with `E` as `Extra`.
    fn replace_extra<E: 'static + Clone>(self, extra: E) -> (Self::Extra, Self::WithExtra<E>);
    /// [`Self::replace_extra`] but discarding [`Self::Extra`].
    fn with_extra<E: 'static + Clone>(self, extra: E) -> Self::WithExtra<E> {
        self.replace_extra(extra).1
    }
    /// [`ParseInput::parse`] with a different `Extra`.
    fn parse_extra<E: 'static + Clone, T: Parse<Self::WithExtra<E>>>(
        self,
        extra: E,
    ) -> crate::Result<T> {
        self.with_extra(extra).parse()
    }
    /// [`ParseInput::parse_inline`] with a different `Extra`.
    fn parse_inline_extra<E: 'static + Clone, T: ParseInline<Self::WithExtra<E>>>(
        &mut self,
        extra: E,
    ) -> crate::Result<T>;
}

impl<T: Sized + IntoIterator> RainbowIterator for T {}

/// This can be parsed by consuming the whole rest of the input.
///
/// Nothing can be parsed after this. It's implementation's responsibility to ensure there are no
/// leftover bytes.
pub trait Parse<I: ParseInput>: Sized {
    /// Parse consuming the whole stream.
    fn parse(input: I) -> crate::Result<Self>;
}

/// This can be parsed from an input, after which we can correctly parse something else.
///
/// When parsed as the last object, makes sure there are no bytes left in the input (fails if there
/// are).
pub trait ParseInline<I: ParseInput>: Parse<I> {
    /// Parse without consuming the whole stream. Errors on unexpected EOF.
    fn parse_inline(input: &mut I) -> crate::Result<Self>;
    /// For implementing [`Parse::parse`].
    fn parse_as_inline(mut input: I) -> crate::Result<Self> {
        let object = Self::parse_inline(&mut input)?;
        input.empty()?;
        Ok(object)
    }
    /// Parse a `Vec` of `Self`. Customisable for optimisations.
    fn parse_vec(input: I) -> crate::Result<Vec<Self>> {
        input.parse_collect()
    }
    /// Parse a `Vec` of `Self` of length `n`. Customisable for optimisations.
    fn parse_vec_n(input: &mut I, n: usize) -> crate::Result<Vec<Self>> {
        (0..n).map(|_| input.parse_inline()).collect()
    }
    /// Parse an array of `Self`. Customisable for optimisations.
    fn parse_array<const N: usize>(input: &mut I) -> crate::Result<[Self; N]> {
        let mut scratch = std::array::from_fn(|_| None);
        for item in scratch.iter_mut() {
            *item = Some(input.parse_inline()?);
        }
        Ok(scratch.map(Option::unwrap))
    }
    /// Parse a [`GenericArray`] of `Self`. Customisable for optimisations.
    fn parse_generic_array<N: ArrayLength>(input: &mut I) -> crate::Result<GenericArray<Self, N>> {
        let mut scratch = GenericArray::default();
        for item in scratch.iter_mut() {
            *item = Some(input.parse_inline()?);
        }
        Ok(scratch.map(Option::unwrap))
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

/// This `Extra` can be used to parse `T` via [`ParseSliceExtra::parse_slice_extra`].
pub trait ExtraFor<T> {
    /// [`ParseSliceExtra::parse_slice_extra`].
    fn parse(&self, data: &[u8], resolve: &Arc<dyn Resolve>) -> Result<T>;

    /// [`Self::parse`], then check that [`FullHash::full_hash`] matches.
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

impl<T: for<'a> Parse<Input<'a, Extra>>, Extra: Clone> ExtraFor<T> for Extra {
    fn parse(&self, data: &[u8], resolve: &Arc<dyn Resolve>) -> Result<T> {
        T::parse_slice_extra(data, resolve, self)
    }
}

impl<T> ToOutput for dyn Send + Sync + ExtraFor<T> {
    fn to_output(&self, _: &mut impl Output) {}
}

impl<T: Tagged> Tagged for dyn Send + Sync + ExtraFor<T> {
    const TAGS: Tags = T::TAGS;
    const HASH: Hash = T::HASH;
}

impl<T> Size for dyn Send + Sync + ExtraFor<T> {
    type Size = typenum::U0;
    const SIZE: usize = 0;
}

impl<T> InlineOutput for dyn Send + Sync + ExtraFor<T> {}
impl<T> ListHashes for dyn Send + Sync + ExtraFor<T> {}
impl<T> Topological for dyn Send + Sync + ExtraFor<T> {}

impl<T, I: PointInput<Extra: Send + Sync + ExtraFor<T>>> Parse<I>
    for Arc<dyn Send + Sync + ExtraFor<T>>
{
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<T, I: PointInput<Extra: Send + Sync + ExtraFor<T>>> ParseInline<I>
    for Arc<dyn Send + Sync + ExtraFor<T>>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Arc::new(input.extra().clone()))
    }
}

impl<T> MaybeHasNiche for dyn Send + Sync + ExtraFor<T> {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}

assert_impl!(
    impl<T, E> Inline<E> for Arc<dyn Send + Sync + ExtraFor<T>>
    where
        T: Object<E>,
        E: 'static + Send + Sync + Clone + ExtraFor<T>,
    {
    }
);

#[doc(hidden)]
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
