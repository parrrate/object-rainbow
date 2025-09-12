extern crate self as object_rainbow;

use std::{
    future::ready,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

pub use anyhow::anyhow;
pub use object_rainbow_derive::{Object, ToOutput};
use sha2::{Digest, Sha256};

mod tuple;

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

pub type Hash = [u8; 32];

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
    index: usize,
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

impl<'a> ReflessInput<'a> {
    fn empty(self) -> crate::Result<()> {
        if self.data.is_empty() {
            Ok(())
        } else {
            Err(Error::ExtraInputLeft)
        }
    }

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

    pub fn parse_inline<T: ReflessInline>(&mut self) -> crate::Result<T> {
        T::parse_inline(self)
    }

    pub fn parse<T: ReflessObject>(self) -> crate::Result<T> {
        T::parse(self)
    }
}

impl Input<'_> {
    fn empty(self) -> crate::Result<()> {
        self.refless.empty()
    }

    fn parse_address(&mut self) -> crate::Result<Address> {
        let hash = *self.parse_chunk()?;
        let index = self.index;
        self.index += 1;
        Ok(Address { hash, index })
    }

    fn parse_point<T: Object>(&mut self) -> crate::Result<Point<T>> {
        let address = self.parse_address()?;
        Ok(Point::from_address(address, self.resolve.clone()))
    }

    fn parse_inline<T: Inline>(&mut self) -> crate::Result<T> {
        T::parse_inline(self)
    }

    fn parse<T: Object>(self) -> crate::Result<T> {
        T::parse(self)
    }
}

pub trait ToOutput {
    fn to_output(&self, output: &mut dyn Output);
}

pub trait Object: 'static + Sized + Send + Sync + ToOutput {
    fn accept_points(&self, visitor: &mut impl PointVisitor);
    fn parse(input: Input) -> crate::Result<Self>;

    fn parse_slice(data: &[u8], resolve: &Arc<dyn Resolve>) -> crate::Result<Self> {
        let input = Input {
            refless: ReflessInput { data, at: 0 },
            resolve,
            index: 0,
        };
        let object = Self::parse(input)?;
        Ok(object)
    }

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        self.accept_points(&mut HashVisitor(|hash| hasher.update(hash)));
        hasher.finalize().into()
    }

    fn tag_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        Self::TAGS.hash(&mut |tag| {
            hasher.update({
                let mut hasher = Sha256::new();
                hasher.update(tag);
                hasher.finalize()
            })
        });
        hasher.finalize().into()
    }

    fn topology(&self) -> TopoVec {
        let mut topolog = TopoVec::new();
        self.accept_points(&mut topolog);
        topolog
    }

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

    fn full_hash(&self) -> Hash {
        let mut output = HashOutput::default();
        output.hasher.update(self.topology_hash());
        output.hasher.update(self.data_hash());
        output.hasher.update(self.tag_hash());
        output.hash()
    }

    const TAGS: Tags = Tags(&[], &[]);
}

pub struct Tags(pub &'static [&'static str], pub &'static [&'static Self]);

impl Tags {
    fn hash(&self, f: &mut impl FnMut(&'static str)) {
        for tag in self.0 {
            f(tag);
        }
        for tags in self.1 {
            tags.hash(f)
        }
    }
}

pub trait Inline: Object {
    fn parse_inline(input: &mut Input) -> crate::Result<Self>;
    fn parse_as_inline(mut input: Input) -> crate::Result<Self> {
        let object = Self::parse_inline(&mut input)?;
        input.empty()?;
        Ok(object)
    }
}

impl<T: Object> Object for Point<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        visitor.visit(self);
    }

    fn parse(input: Input) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<T> ToOutput for Point<T> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&self.hash);
    }
}

impl<T: Object> Inline for Point<T> {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        input.parse_point()
    }
}

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

pub trait ReflessObject: 'static + Sized + Send + Sync + ToOutput {
    fn parse(input: ReflessInput) -> crate::Result<Self>;

    fn parse_slice(data: &[u8]) -> crate::Result<Self> {
        let input = ReflessInput { data, at: 0 };
        let object = Self::parse(input)?;
        Ok(object)
    }
}

pub trait ReflessInline: ReflessObject {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self>;
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

impl<T: ReflessObject> Object for Refless<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        let _ = visitor;
    }

    fn parse(input: Input) -> crate::Result<Self> {
        T::parse(input.refless).map(Self)
    }
}

impl<T: ReflessInline> Inline for Refless<T> {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        T::parse_inline(input).map(Self)
    }
}

impl ToOutput for () {
    fn to_output(&self, _: &mut dyn Output) {}
}

impl Object for () {
    fn accept_points(&self, _: &mut impl PointVisitor) {}

    fn parse(input: Input) -> crate::Result<Self> {
        Inline::parse_as_inline(input)
    }
}

impl Inline for () {
    fn parse_inline(_: &mut Input) -> crate::Result<Self> {
        Ok(())
    }
}

impl ReflessObject for () {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        ReflessInline::parse_as_inline(input)
    }
}

impl ReflessInline for () {
    fn parse_inline(_: &mut ReflessInput) -> crate::Result<Self> {
        Ok(())
    }
}

impl<T: ToOutput> ToOutput for (T,) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
    }
}

impl<T: Object> Object for (T,) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
    }

    fn parse(input: Input) -> crate::Result<Self> {
        Ok((input.parse()?,))
    }

    const TAGS: Tags = T::TAGS;
}

impl<T: Inline> Inline for (T,) {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        Ok((input.parse_inline()?,))
    }
}

impl<T: ReflessObject> ReflessObject for (T,) {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        Ok((input.parse()?,))
    }
}

impl<T: ReflessInline> ReflessInline for (T,) {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        Ok((input.parse_inline()?,))
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

impl<const N: usize> ToOutput for [u8; N] {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self);
    }
}

impl<const N: usize> ReflessObject for [u8; N] {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<const N: usize> ReflessInline for [u8; N] {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        input.parse_chunk().copied()
    }
}

impl<T: ToOutput> ToOutput for Arc<T> {
    fn to_output(&self, output: &mut dyn Output) {
        (**self).to_output(output);
    }
}

impl<T: Object> Object for Arc<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        (**self).accept_points(visitor);
    }

    fn parse(input: Input) -> crate::Result<Self> {
        T::parse(input).map(Self::new)
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn topology(&self) -> TopoVec {
        (**self).topology()
    }

    fn full_hash(&self) -> Hash {
        (**self).full_hash()
    }

    const TAGS: Tags = T::TAGS;
}

impl ToOutput for Vec<u8> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(self);
    }
}

impl ReflessObject for Vec<u8> {
    fn parse(input: ReflessInput) -> crate::Result<Self> {
        Ok(input.parse_all()?.into())
    }
}

pub trait Size {
    const SIZE: usize;
}

pub trait Fixed: Size + Inline {}

impl<T: Size + Inline> Fixed for T {}

pub trait ReflessFixed: Size + ReflessInline {}

impl<T: Size + ReflessInline> ReflessFixed for T {}

impl<T: Size, const N: usize> Size for [T; N] {
    const SIZE: usize = T::SIZE * N;
}

impl<const N: usize> Size for [u8; N] {
    const SIZE: usize = N;
}

#[derive(ToOutput, Object)]
pub struct DeriveExample<A, B>(A, B);
