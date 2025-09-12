extern crate self as object_rainbow;

use std::{
    future::ready,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

pub use anyhow::anyhow;
pub use object_rainbow_derive::{
    Inline, Object, Parse, ParseAsInline, ParseInline, ReflessInline, ReflessObject, Size, Tagged,
    ToOutput, Topological,
};
use sha2::{Digest, Sha256};

mod impls;
pub mod numeric;
mod sha2_const;

#[macro_export]
macro_rules! error_parse {
    ($($t:tt)*) => {
        $crate::Error::Parse(::anyhow::anyhow!($($t)*))
    };
}

#[macro_export]
macro_rules! error_fetch {
    ($($t:tt)*) => {
        $crate::Error::Fetch(::anyhow::anyhow!($($t)*))
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("extra input left")]
    ExtraInputLeft,
    #[error("end of input")]
    EndOfInput,
    #[error("address index out of bounds")]
    AddressOutOfBounds,
    #[error("hash resolution mismatch")]
    HashMismatch,
    #[error(transparent)]
    Parse(anyhow::Error),
    #[error(transparent)]
    Fetch(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub const HASH_SIZE: usize = sha2_const::Sha256::DIGEST_SIZE;

pub type Hash = [u8; HASH_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub index: usize,
    pub hash: Hash,
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
    fn fetch(&self) -> FailFuture<Self::T>;
}

#[derive(ParseAsInline)]
pub struct Point<T> {
    hash: Hash,
    origin: Arc<dyn Fetch<T = T>>,
}

impl<T> Clone for Point<T> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            origin: self.origin.clone(),
        }
    }
}

impl<T> Size for Point<T> {
    const SIZE: usize = HASH_SIZE;
}

impl<T: Object> Point<T> {
    pub fn from_address(address: Address, resolve: Arc<dyn Resolve>) -> Self {
        Self {
            hash: address.hash,
            origin: Arc::new(ByAddress {
                address,
                resolve,
                _object: PhantomData,
            }),
        }
    }
}

struct ByAddress<T> {
    address: Address,
    resolve: Arc<dyn Resolve>,
    _object: PhantomData<T>,
}

impl<T: Object> Fetch for ByAddress<T> {
    type T = T;

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(async {
            let (data, resolve) = self.resolve.resolve(self.address).await?;
            let object = T::parse_slice(&data, &resolve)?;
            Ok(object)
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

pub struct HashVisitor<F>(F);

impl<F: FnMut(Hash)> PointVisitor for HashVisitor<F> {
    fn visit<T: Object>(&mut self, point: &Point<T>) {
        self.0(point.hash);
    }
}

pub struct ReflessInput<'a> {
    data: &'a [u8],
    at: usize,
}

pub struct Input<'a> {
    refless: ReflessInput<'a>,
    resolve: &'a Arc<dyn Resolve>,
    index: &'a mut usize,
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

impl<'a> ReflessInput<'a> {
    pub fn parse_chunk<const N: usize>(&mut self) -> crate::Result<&'a [u8; N]> {
        match self.data.split_first_chunk() {
            Some((chunk, data)) => {
                self.data = data;
                self.at += N;
                Ok(chunk)
            }
            None => Err(Error::EndOfInput),
        }
    }

    pub fn parse_n(&mut self, n: usize) -> crate::Result<&'a [u8]> {
        match self.data.split_at_checked(n) {
            Some((chunk, data)) => {
                self.data = data;
                self.at += n;
                Ok(chunk)
            }
            None => Err(Error::EndOfInput),
        }
    }

    pub fn parse_all(self) -> crate::Result<&'a [u8]> {
        Ok(self.data)
    }

    pub fn tell(&self) -> usize {
        self.at
    }
}

impl ParseInput for Input<'_> {
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
        let index = *self.index;
        *self.index += 1;
        Ok(Address { hash, index })
    }

    fn parse_point<T: Object>(&mut self) -> crate::Result<Point<T>> {
        let address = self.parse_address()?;
        Ok(Point::from_address(address, self.resolve.clone()))
    }
}

pub trait ToOutput {
    fn to_output(&self, output: &mut dyn Output);

    fn output<T: Output + Default>(&self) -> T {
        let mut output = T::default();
        self.to_output(&mut output);
        output
    }

    fn data_hash(&self) -> Hash {
        let mut output = HashOutput::default();
        self.to_output(&mut output);
        output.hash()
    }
}

pub trait Topological {
    fn accept_points(&self, visitor: &mut impl PointVisitor);

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
            refless: ReflessInput { data, at: 0 },
            resolve,
            index: &mut 0,
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

impl<T> Tagged for Point<T> {}

impl<T: Object> Object for Point<T> {}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&self.hash);
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
        &self.hash
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
        let input = ReflessInput { data, at: 0 };
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

impl<T> Topological for Refless<T> {
    fn accept_points(&self, _: &mut impl PointVisitor) {}
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

impl<T: Object + Clone> Point<T> {
    pub fn from_object(object: T) -> Self {
        Self {
            hash: object.full_hash(),
            origin: Arc::new(LocalOrigin(object)),
        }
    }
}

struct LocalOrigin<T>(T);

impl<T: Object + Clone> Fetch for LocalOrigin<T> {
    type T = T;

    fn fetch(&self) -> FailFuture<Self::T> {
        Box::pin(ready(Ok(self.0.clone())))
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
            Err(Error::HashMismatch)
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

impl<T> Fetch for Point<T> {
    type T = T;

    fn fetch(&self) -> FailFuture<Self::T> {
        self.origin.fetch()
    }
}

pub trait Size {
    const SIZE: usize;
}

pub trait Fixed: Size + Inline {}

impl<T: Size + Inline> Fixed for T {}

pub trait ReflessFixed: Size + ReflessInline {}

impl<T: Size + ReflessInline> ReflessFixed for T {}

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
