extern crate self as object_rainbow;

use std::{
    cell::Cell,
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
pub use self::niche::{
    AutoEnumNiche, HackNiche, MaybeHasNiche, Niche, NoNiche, SomeNiche, ZeroNiche, ZeroNoNiche,
};
#[doc(hidden)]
pub use self::niche::{MaybeNiche, MnArray, NicheFoldOrArray, NicheOr};

pub mod enumkind;
mod impls;
pub mod length_prefixed;
mod niche;
pub mod numeric;
mod sha2_const;

#[macro_export]
macro_rules! error_parse {
    ($($t:tt)*) => {
        $crate::Error::Parse($crate::anyhow!($($t)*))
    };
}

#[macro_export]
macro_rules! error_fetch {
    ($($t:tt)*) => {
        $crate::Error::Fetch($crate::anyhow!($($t)*))
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(anyhow::Error),
    #[error(transparent)]
    Fetch(anyhow::Error),
    #[error("extra input left")]
    ExtraInputLeft,
    #[error("end of input")]
    EndOfInput,
    #[error("address index out of bounds")]
    AddressOutOfBounds,
    #[error("hash resolution mismatch")]
    ResolutionMismatch,
    #[error("data hash mismatch")]
    DataMismatch,
    #[error("discriminant overflow")]
    DiscriminantOverflow,
    #[error("zero")]
    Zero,
    #[error("out of bounds")]
    OutOfBounds,
    #[error("length out of bounds")]
    LenOutOfBounds,
    #[error(transparent)]
    Utf8(std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub const HASH_SIZE: usize = sha2_const::Sha256::DIGEST_SIZE;

pub type Hash = [u8; HASH_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub index: usize,
    pub hash: Hash,
}

#[derive(Debug, Clone, Copy)]
struct OptionalHash(Hash);

impl From<Hash> for OptionalHash {
    fn from(hash: Hash) -> Self {
        Self(hash)
    }
}

impl OptionalHash {
    fn get(&self) -> Option<&Hash> {
        (self.0 != Hash::default()).then_some(&self.0)
    }

    fn unwrap(&self) -> &Hash {
        self.get().unwrap()
    }

    fn clear(&mut self) {
        self.0 = Hash::default();
    }
}

pub type FailFuture<'a, T> = Pin<Box<dyn 'a + Send + Future<Output = Result<T>>>>;

pub type ByteNode = (Vec<u8>, Arc<dyn Resolve>);

pub trait Resolve: Send + Sync {
    fn resolve(&self, address: Address) -> FailFuture<ByteNode>;
}

pub trait FetchBytes {
    fn fetch_bytes(&self) -> FailFuture<ByteNode>;
}

pub trait Fetch: Send + Sync + FetchBytes {
    type T;
    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)>;
    fn fetch(&self) -> FailFuture<Self::T>;
    fn get(&self) -> Option<&Self::T> {
        None
    }
    fn get_mut(&mut self) -> Option<&mut Self::T> {
        None
    }
    fn get_mut_finalize(&mut self) {}
}

#[derive(ParseAsInline)]
pub struct Point<T> {
    hash: OptionalHash,
    origin: Arc<dyn Fetch<T = T>>,
}

impl<T> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash().cmp(other.hash())
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
            origin: self.origin.clone(),
        }
    }
}

impl<T> Point<T> {
    fn from_origin(hash: Hash, origin: Arc<dyn Fetch<T = T>>) -> Self {
        Self {
            hash: hash.into(),
            origin,
        }
    }
}

impl<T> Size for Point<T> {
    const SIZE: usize = HASH_SIZE;
    type Size = typenum::generic_const_mappings::U<HASH_SIZE>;
}

impl<T: Object> Point<T> {
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self::from_origin(
            address.hash,
            Arc::new(ByAddress {
                address,
                resolve,
                _object: PhantomData,
            }),
        )
    }
}

struct ByAddress<T> {
    address: Address,
    resolve: Arc<dyn Resolve>,
    _object: PhantomData<T>,
}

impl<T: Object> Fetch for ByAddress<T> {
    type T = T;

    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)> {
        Box::pin(async {
            let (data, resolve) = self.resolve.resolve(self.address).await?;
            let object = T::parse_slice(&data, &resolve)?;
            if self.address.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok((object, resolve))
            }
        })
    }

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(async {
            let (data, resolve) = self.resolve.resolve(self.address).await?;
            let object = T::parse_slice(&data, &resolve)?;
            if self.address.hash != object.full_hash() {
                Err(Error::DataMismatch)
            } else {
                Ok(object)
            }
        })
    }
}

impl<T> FetchBytes for ByAddress<T> {
    fn fetch_bytes(&self) -> FailFuture<ByteNode> {
        self.resolve.resolve(self.address)
    }
}

pub trait PointVisitor {
    fn visit<T: Object>(&mut self, point: &Point<T>);
}

struct HashVisitor<F>(F);

impl<F: FnMut(Hash)> PointVisitor for HashVisitor<F> {
    fn visit<T: Object>(&mut self, point: &Point<T>) {
        self.0(*point.hash());
    }
}

pub struct ReflessInput<'a> {
    data: &'a [u8],
}

pub struct Input<'a> {
    refless: ReflessInput<'a>,
    resolve: &'a Arc<dyn Resolve>,
    index: &'a Cell<usize>,
}

impl<'a> Deref for Input<'a> {
    type Target = ReflessInput<'a>;

    fn deref(&self) -> &Self::Target {
        &self.refless
    }
}

impl DerefMut for Input<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.refless
    }
}

impl ParseInput for ReflessInput<'_> {
    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a,
    {
        match self.data.split_first_chunk() {
            Some((chunk, data)) => {
                self.data = data;
                Ok(chunk)
            }
            None => Err(Error::EndOfInput),
        }
    }

    fn parse_n<'a>(&mut self, n: usize) -> crate::Result<&'a [u8]>
    where
        Self: 'a,
    {
        match self.data.split_at_checked(n) {
            Some((chunk, data)) => {
                self.data = data;
                Ok(chunk)
            }
            None => Err(Error::EndOfInput),
        }
    }

    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T> {
        let input = ReflessInput {
            data: self.parse_n(n)?,
        };
        T::parse(input)
    }

    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>> {
        match self.data.split_at_checked(n) {
            Some((chunk, data)) => {
                if chunk == c {
                    self.data = data;
                    Ok(None)
                } else {
                    self.parse_inline().map(Some)
                }
            }
            None => Err(Error::EndOfInput),
        }
    }

    fn parse_all<'a>(self) -> &'a [u8]
    where
        Self: 'a,
    {
        self.data
    }

    fn empty(self) -> crate::Result<()> {
        if self.data.is_empty() {
            Ok(())
        } else {
            Err(Error::ExtraInputLeft)
        }
    }

    fn non_empty(self) -> Option<Self> {
        if self.data.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

impl ParseInput for Input<'_> {
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
                data: self.parse_n(n)?,
            },
            resolve: self.resolve,
            index: self.index,
        };
        T::parse(input)
    }

    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>> {
        match self.data.split_at_checked(n) {
            Some((chunk, data)) => {
                if chunk == c {
                    self.data = data;
                    Ok(None)
                } else {
                    self.parse_inline().map(Some)
                }
            }
            None => Err(Error::EndOfInput),
        }
    }

    fn parse_all<'a>(self) -> &'a [u8]
    where
        Self: 'a,
    {
        self.refless.parse_all()
    }

    fn empty(self) -> crate::Result<()> {
        self.refless.empty()
    }

    fn non_empty(mut self) -> Option<Self> {
        self.refless = self.refless.non_empty()?;
        Some(self)
    }
}

impl Input<'_> {
    fn parse_address(&mut self) -> crate::Result<Address> {
        let hash = *self.parse_chunk()?;
        let index = self.index.get();
        self.index.set(index + 1);
        Ok(Address { hash, index })
    }

    fn parse_point<T: Object>(&mut self) -> crate::Result<Point<T>> {
        let address = self.parse_address()?;
        Ok(Point::from_address(address, self.resolve.clone()))
    }
}

pub trait ToOutput {
    fn to_output(&self, output: &mut dyn Output);

    fn data_hash(&self) -> Hash {
        let mut output = HashOutput::default();
        self.to_output(&mut output);
        output.hash()
    }
}

pub trait ToOutputExt: ToOutput {
    fn output<T: Output + Default>(&self) -> T {
        let mut output = T::default();
        self.to_output(&mut output);
        output
    }
}

impl<T: ?Sized + ToOutput> ToOutputExt for T {}

pub trait Topological {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        let _ = visitor;
    }

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        self.accept_points(&mut HashVisitor(|hash| hasher.update(hash)));
        hasher.finalize().into()
    }

    fn topology(&self) -> TopoVec {
        let mut topolog = TopoVec::new();
        self.accept_points(&mut topolog);
        topolog
    }
}

pub trait Tagged {
    const TAGS: Tags = Tags(&[], &[]);

    const HASH: Hash = const { Self::TAGS.const_hash(sha2_const::Sha256::new()).finalize() };
}

pub trait Object:
    'static + Sized + Send + Sync + ToOutput + Topological + Tagged + for<'a> Parse<Input<'a>>
{
    fn parse_slice(data: &[u8], resolve: &Arc<dyn Resolve>) -> crate::Result<Self> {
        let input = Input {
            refless: ReflessInput { data },
            resolve,
            index: &Cell::new(0),
        };
        let object = Self::parse(input)?;
        Ok(object)
    }

    fn full_hash(&self) -> Hash {
        let mut output = HashOutput::default();
        output.hasher.update(Self::HASH);
        output.hasher.update(self.topology_hash());
        output.hasher.update(self.data_hash());
        output.hash()
    }
}

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
            while i < self.0.len() {
                hasher = self.1[i].const_hash(hasher);
                i += 1;
            }
        }
        hasher
    }
}

pub trait Inline: Object + for<'a> ParseInline<Input<'a>> {}

impl<T: Object> Topological for Point<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }
}

impl<T: Object> ParseInline<Input<'_>> for Point<T> {
    fn parse_inline(input: &mut Input<'_>) -> crate::Result<Self> {
        input.parse_point()
    }
}

impl<T: Tagged> Tagged for Point<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T: Object> Object for Point<T> {}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self.hash());
    }
}

impl<T: Object> Inline for Point<T> {}

pub trait Topology: Send + Sync {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Arc<dyn Singular>>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Singular: Send + Sync + FetchBytes {
    fn hash(&self) -> &Hash;
}

pub type TopoVec = Vec<Arc<dyn Singular>>;

impl PointVisitor for TopoVec {
    fn visit<T: Object>(&mut self, point: &Point<T>) {
        self.push(Arc::new(point.clone()));
    }
}

impl<T> FetchBytes for Point<T> {
    fn fetch_bytes(&self) -> FailFuture<ByteNode> {
        self.origin.fetch_bytes()
    }
}

impl<T> Singular for Point<T> {
    fn hash(&self) -> &Hash {
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

pub trait ReflessObject:
    'static + Sized + Send + Sync + ToOutput + Tagged + for<'a> Parse<ReflessInput<'a>>
{
    fn parse_slice(data: &[u8]) -> crate::Result<Self> {
        let input = ReflessInput { data };
        let object = Self::parse(input)?;
        Ok(object)
    }
}

pub trait ReflessInline: ReflessObject + for<'a> ParseInline<ReflessInput<'a>> {
    fn parse_as_inline(mut input: ReflessInput) -> crate::Result<Self> {
        let object = Self::parse_inline(&mut input)?;
        input.empty()?;
        Ok(object)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Refless<T>(pub T);

impl<T> Deref for Refless<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Refless<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ToOutput> ToOutput for Refless<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
    }
}

impl<'a, T: Parse<ReflessInput<'a>>> Parse<Input<'a>> for Refless<T> {
    fn parse(input: Input<'a>) -> crate::Result<Self> {
        T::parse(input.refless).map(Self)
    }
}

impl<'a, T: ParseInline<ReflessInput<'a>>> ParseInline<Input<'a>> for Refless<T> {
    fn parse_inline(input: &mut Input<'a>) -> crate::Result<Self> {
        T::parse_inline(input).map(Self)
    }
}

impl<T: Tagged> Tagged for Refless<T> {
    const TAGS: Tags = T::TAGS;
}

impl<T> Topological for Refless<T> {}
impl<T: ReflessObject> Object for Refless<T> {}
impl<T: ReflessInline> Inline for Refless<T> {}

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
        self.hasher.finalize().into()
    }
}

pub struct PointMut<'a, T: Object> {
    hash: &'a mut OptionalHash,
    origin: &'a mut dyn Fetch<T = T>,
}

impl<T: Object> Deref for PointMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.origin.get().unwrap()
    }
}

impl<T: Object> DerefMut for PointMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.origin.get_mut().unwrap()
    }
}

impl<T: Object> Drop for PointMut<'_, T> {
    fn drop(&mut self) {
        self.origin.get_mut_finalize();
        self.hash.0 = self.full_hash();
    }
}

impl<T: Object + Clone> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self::from_origin(object.full_hash(), Arc::new(LocalOrigin(object)))
    }

    fn yolo_mut(&mut self) -> bool {
        self.origin.get().is_some()
            && Arc::get_mut(&mut self.origin).is_some_and(|origin| origin.get_mut().is_some())
    }

    pub async fn fetch_mut(&mut self) -> crate::Result<PointMut<T>> {
        if !self.yolo_mut() {
            let object = self.origin.fetch().await?;
            self.origin = Arc::new(LocalOrigin(object));
        }
        let origin = Arc::get_mut(&mut self.origin).unwrap();
        assert!(origin.get_mut().is_some());
        self.hash.clear();
        Ok(PointMut {
            hash: &mut self.hash,
            origin,
        })
    }
}

struct LocalOrigin<T>(T);

impl<T> Deref for LocalOrigin<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LocalOrigin<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Object + Clone> Fetch for LocalOrigin<T> {
    type T = T;

    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)> {
        Box::pin(ready(Ok((
            self.0.clone(),
            Arc::new(ByTopology {
                topology: self.0.topology(),
            }) as _,
        ))))
    }

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(ready(Ok(self.0.clone())))
    }

    fn get(&self) -> Option<&Self::T> {
        Some(self)
    }

    fn get_mut(&mut self) -> Option<&mut Self::T> {
        Some(self)
    }
}

impl<T: Object> FetchBytes for LocalOrigin<T> {
    fn fetch_bytes(&self) -> FailFuture<ByteNode> {
        Box::pin(ready(Ok((
            self.0.output(),
            Arc::new(ByTopology {
                topology: self.0.topology(),
            }) as _,
        ))))
    }
}

struct ByTopology {
    topology: TopoVec,
}

impl ByTopology {
    fn try_resolve(&self, address: Address) -> Result<FailFuture<ByteNode>> {
        let point = self
            .topology
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        if *point.hash() != address.hash {
            Err(Error::ResolutionMismatch)
        } else {
            Ok(point.fetch_bytes())
        }
    }
}

impl Resolve for ByTopology {
    fn resolve(&self, address: Address) -> FailFuture<ByteNode> {
        self.try_resolve(address)
            .map_err(Err)
            .map_err(ready)
            .map_err(Box::pin)
            .unwrap_or_else(|x| x)
    }
}

impl<T: Object> Fetch for Point<T> {
    type T = T;

    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)> {
        self.origin.fetch_full()
    }

    fn fetch(&self) -> FailFuture<Self::T> {
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
        self.hash.0 = origin.get().unwrap().full_hash();
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
    fn parse_chunk<'a, const N: usize>(&mut self) -> crate::Result<&'a [u8; N]>
    where
        Self: 'a;
    fn parse_n<'a>(&mut self, n: usize) -> crate::Result<&'a [u8]>
    where
        Self: 'a;
    fn parse_ahead<T: Parse<Self>>(&mut self, n: usize) -> crate::Result<T>;
    fn parse_compare<T: ParseInline<Self>>(&mut self, n: usize, c: &[u8]) -> Result<Option<T>>;
    fn parse_all<'a>(self) -> &'a [u8]
    where
        Self: 'a;
    fn empty(self) -> crate::Result<()>;
    fn non_empty(self) -> Option<Self>;

    fn consume(self, f: impl FnMut(&mut Self) -> crate::Result<()>) -> crate::Result<()> {
        self.collect(f)
    }

    fn parse_collect<T: ParseInline<Self>, B: FromIterator<T>>(self) -> crate::Result<B> {
        self.collect(|input| input.parse_inline())
    }

    fn collect<T, B: FromIterator<T>>(self, f: impl FnMut(&mut Self) -> T) -> B {
        self.iter(f).collect()
    }

    fn iter<T>(self, mut f: impl FnMut(&mut Self) -> T) -> impl Iterator<Item = T> {
        let mut state = Some(self);
        std::iter::from_fn(move || {
            let mut input = state.take()?.non_empty()?;
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
}

pub trait Equivalent<T>: Sized {
    fn into_equivalent(self) -> T;
    fn from_equivalent(object: T) -> Self;
}

impl<U: 'static + Equivalent<T>, T: 'static> Equivalent<Point<T>> for Point<U> {
    fn into_equivalent(self) -> Point<T> {
        Point {
            hash: self.hash,
            origin: Arc::new(MapEquivalent {
                origin: self.origin,
                map: U::into_equivalent,
            }),
        }
    }

    fn from_equivalent(object: Point<T>) -> Self {
        Point {
            hash: object.hash,
            origin: Arc::new(MapEquivalent {
                origin: object.origin,
                map: U::from_equivalent,
            }),
        }
    }
}

struct MapEquivalent<T, F> {
    origin: Arc<dyn Fetch<T = T>>,
    map: F,
}

impl<T, F> FetchBytes for MapEquivalent<T, F> {
    fn fetch_bytes(&self) -> FailFuture<ByteNode> {
        self.origin.fetch_bytes()
    }
}

trait Map1<T>: Fn(T) -> Self::U {
    type U;
}

impl<T, U, F: Fn(T) -> U> Map1<T> for F {
    type U = U;
}

impl<T, F: 'static + Send + Sync + Map1<T>> Fetch for MapEquivalent<T, F> {
    type T = F::U;

    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
    }
}

impl<T: 'static + ToOutput> Point<T> {
    pub fn map<U>(self, f: impl 'static + Send + Sync + Fn(T) -> U) -> Point<U> {
        Point {
            hash: self.hash,
            origin: Arc::new(Map {
                origin: self.origin,
                map: f,
            }),
        }
    }
}

struct Map<T, F> {
    origin: Arc<dyn Fetch<T = T>>,
    map: F,
}

impl<T: ToOutput, F> FetchBytes for Map<T, F> {
    fn fetch_bytes(&self) -> FailFuture<ByteNode> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| (x.output(), r)))
    }
}

impl<T: ToOutput, F: 'static + Send + Sync + Map1<T>> Fetch for Map<T, F> {
    type T = F::U;

    fn fetch_full(&self) -> FailFuture<(Self::T, Arc<dyn Resolve>)> {
        Box::pin(self.origin.fetch_full().map_ok(|(x, r)| ((self.map)(x), r)))
    }

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(self.origin.fetch().map_ok(&self.map))
    }
}

impl<T> MaybeHasNiche for Point<T> {
    type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
}

#[test]
fn options() {
    type T0 = bool;
    type T1 = Option<T0>;
    type T2 = Option<T1>;
    type T3 = Option<T2>;
    type T4 = Option<T3>;
    type T5 = Option<T4>;
    assert_eq!(T0::SIZE, 1);
    assert_eq!(T1::SIZE, 1);
    assert_eq!(T2::SIZE, 2);
    assert_eq!(T3::SIZE, 2);
    assert_eq!(T4::SIZE, 3);
    assert_eq!(T5::SIZE, 3);
    assert_eq!(false.output::<Vec<u8>>(), [0]);
    assert_eq!(true.output::<Vec<u8>>(), [1]);
    assert_eq!(Some(false).output::<Vec<u8>>(), [0]);
    assert_eq!(Some(true).output::<Vec<u8>>(), [1]);
    assert_eq!(None::<bool>.output::<Vec<u8>>(), [2]);
    assert_eq!(Some(Some(false)).output::<Vec<u8>>(), [0, 0]);
    assert_eq!(Some(Some(true)).output::<Vec<u8>>(), [0, 1]);
    assert_eq!(Some(None::<bool>).output::<Vec<u8>>(), [0, 2]);
    assert_eq!(None::<Option<bool>>.output::<Vec<u8>>(), [1, 0]);
    assert_eq!(Some(Some(Some(false))).output::<Vec<u8>>(), [0, 0]);
    assert_eq!(Some(Some(Some(true))).output::<Vec<u8>>(), [0, 1]);
    assert_eq!(Some(Some(None::<bool>)).output::<Vec<u8>>(), [0, 2]);
    assert_eq!(Some(None::<Option<bool>>).output::<Vec<u8>>(), [1, 0]);
    assert_eq!(None::<Option<Option<bool>>>.output::<Vec<u8>>(), [2, 0]);
    assert_eq!(Option::<Point<()>>::SIZE, HASH_SIZE);
    assert_eq!(Some(()).output::<Vec<u8>>(), [0]);
    assert_eq!(Some(((), ())).output::<Vec<u8>>(), [0]);
    assert_eq!(Some(((), true)).output::<Vec<u8>>(), [1]);
    assert_eq!(Some((true, true)).output::<Vec<u8>>(), [1, 1]);
    assert_eq!(Some((Some(true), true)).output::<Vec<u8>>(), [1, 1]);
    assert_eq!(Some((None::<bool>, true)).output::<Vec<u8>>(), [2, 1]);
    assert_eq!(Some((true, None::<bool>)).output::<Vec<u8>>(), [1, 2]);
    assert_eq!(None::<(Option<bool>, bool)>.output::<Vec<u8>>(), [0, 2]);
    assert_eq!(None::<(bool, Option<bool>)>.output::<Vec<u8>>(), [2, 0]);
    assert_eq!(
        Some(Some((Some(true), Some(true)))).output::<Vec<u8>>(),
        [0, 1, 1],
    );
}
