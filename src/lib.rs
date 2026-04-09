use std::{
    future::ready,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

use sha2::{Digest, Sha256};

mod tuple;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not empty")]
    NotEmpty,
    #[error("eof")]
    Eof,
    #[error("out of bounds")]
    OutOfBounds,
    #[error("mismatch")]
    Mismatch,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type Hash = [u8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub index: usize,
    pub hash: Hash,
}

pub type FailFuture<'a, T> = Pin<Box<dyn 'a + Send + Future<Output = Result<T>>>>;

pub type ResolvedBytes = (Vec<u8>, Arc<dyn Resolver>);

pub trait Resolver: Send + Sync {
    fn resolve(&self, address: Address) -> FailFuture<ResolvedBytes>;
}

pub trait Origin: Send + Sync + ResolveBytes {
    type T;
    fn resolve(&self) -> FailFuture<Self::T>;
}

pub struct Ref<T> {
    hash: Hash,
    origin: Arc<dyn Origin<T = T>>,
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            origin: self.origin.clone(),
        }
    }
}

impl<T: Object> Ref<T> {
    pub fn from_address(address: Address, resolver: Arc<dyn Resolver>) -> Self {
        Self {
            hash: address.hash,
            origin: Arc::new(ResolverOrigin {
                address,
                resolver,
                _object: PhantomData,
            }),
        }
    }
}

struct ResolverOrigin<T> {
    address: Address,
    resolver: Arc<dyn Resolver>,
    _object: PhantomData<T>,
}

impl<T: Object> Origin for ResolverOrigin<T> {
    type T = T;

    fn resolve(&self) -> FailFuture<Self::T> {
        Box::pin(async {
            let (data, resolver) = self.resolver.resolve(self.address).await?;
            let object = T::parse_slice(&data, &resolver)?;
            Ok(object)
        })
    }
}

impl<T> ResolveBytes for ResolverOrigin<T> {
    fn resolve_bytes(&self) -> FailFuture<ResolvedBytes> {
        self.resolver.resolve(self.address)
    }
}

pub trait RefVisitor {
    fn visit<T: Object>(&mut self, point: &Ref<T>);
}

pub struct HashVisitor<F>(F);

impl<F: FnMut(Hash)> RefVisitor for HashVisitor<F> {
    fn visit<T: Object>(&mut self, point: &Ref<T>) {
        self.0(point.hash)
    }
}

pub struct ReflessInput<'a> {
    data: &'a [u8],
    at: usize,
}

pub struct Input<'a> {
    refless: ReflessInput<'a>,
    resolver: &'a Arc<dyn Resolver>,
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
            Err(Error::NotEmpty)
        }
    }

    pub fn parse_chunk<const N: usize>(&mut self) -> crate::Result<&'a [u8; N]> {
        match self.data.split_first_chunk() {
            Some((chunk, data)) => {
                self.data = data;
                self.at += N;
                Ok(chunk)
            }
            None => Err(Error::Eof),
        }
    }

    pub fn parse_n(&mut self, n: usize) -> crate::Result<&'a [u8]> {
        match self.data.split_at_checked(n) {
            Some((chunk, data)) => {
                self.data = data;
                self.at += n;
                Ok(chunk)
            }
            None => Err(Error::Eof),
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

    fn parse_ref<T: Object>(&mut self) -> crate::Result<Ref<T>> {
        let address = self.parse_address()?;
        Ok(Ref::from_address(address, self.resolver.clone()))
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
    fn accept_refs(&self, visitor: &mut impl RefVisitor);
    fn parse(input: Input) -> crate::Result<Self>;

    fn parse_slice(data: &[u8], resolver: &Arc<dyn Resolver>) -> crate::Result<Self> {
        let input = Input {
            refless: ReflessInput { data, at: 0 },
            resolver,
            index: 0,
        };
        let object = Self::parse(input)?;
        Ok(object)
    }

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        self.accept_refs(&mut HashVisitor(|hash| hasher.update(hash)));
        hasher.finalize().into()
    }

    fn topology(&self) -> TopoVec {
        let mut topolog = TopoVec::new();
        self.accept_refs(&mut topolog);
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
        output.hash()
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

impl<T: Object> Object for Ref<T> {
    fn accept_refs(&self, visitor: &mut impl RefVisitor) {
        visitor.visit(self);
    }

    fn parse(input: Input) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<T> ToOutput for Ref<T> {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&self.hash);
    }
}

impl<T: Object> Inline for Ref<T> {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        input.parse_ref()
    }
}

pub trait Topology: Send + Sync {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Arc<dyn Singular>>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait ResolveBytes {
    fn resolve_bytes(&self) -> FailFuture<ResolvedBytes>;
}

pub trait Singular: Send + Sync + ResolveBytes {
    fn hash(&self) -> &Hash;
}

pub type TopoVec = Vec<Arc<dyn Singular>>;

impl RefVisitor for TopoVec {
    fn visit<T: Object>(&mut self, point: &Ref<T>) {
        self.push(Arc::new(point.clone()));
    }
}

impl<T> ResolveBytes for Ref<T> {
    fn resolve_bytes(&self) -> FailFuture<ResolvedBytes> {
        self.origin.resolve_bytes()
    }
}

impl<T> Singular for Ref<T> {
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
    fn accept_refs(&self, visitor: &mut impl RefVisitor) {
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
    fn accept_refs(&self, _: &mut impl RefVisitor) {}

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
    fn accept_refs(&self, visitor: &mut impl RefVisitor) {
        self.0.accept_refs(visitor);
    }

    fn parse(input: Input) -> crate::Result<Self> {
        Ok((input.parse()?,))
    }
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
    fn tell(&self) -> usize;
}

impl Output for Vec<u8> {
    fn write(&mut self, data: &[u8]) {
        self.extend_from_slice(data);
    }

    fn tell(&self) -> usize {
        self.len()
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

    fn tell(&self) -> usize {
        self.at
    }
}

impl HashOutput {
    fn hash(self) -> Hash {
        self.hasher.finalize().into()
    }
}

impl<T: Object + Clone> Ref<T> {
    pub fn from_object(object: T) -> Self {
        Self {
            hash: object.full_hash(),
            origin: Arc::new(LocalOrigin(object)),
        }
    }
}

struct LocalOrigin<T>(T);

impl<T: Object + Clone> Origin for LocalOrigin<T> {
    type T = T;

    fn resolve(&self) -> FailFuture<Self::T> {
        Box::pin(ready(Ok(self.0.clone())))
    }
}

impl<T: Object> ResolveBytes for LocalOrigin<T> {
    fn resolve_bytes(&self) -> FailFuture<ResolvedBytes> {
        Box::pin(ready(Ok((
            self.0.output(),
            Arc::new(SingularResolver {
                topology: self.0.topology(),
            }) as _,
        ))))
    }
}

struct SingularResolver {
    topology: TopoVec,
}

impl SingularResolver {
    fn try_resolve(&self, address: Address) -> Result<FailFuture<ResolvedBytes>> {
        let point = self.topology.get(address.index).ok_or(Error::OutOfBounds)?;
        if *point.hash() != address.hash {
            Err(Error::Mismatch)
        } else {
            Ok(point.resolve_bytes())
        }
    }
}

impl Resolver for SingularResolver {
    fn resolve(&self, address: Address) -> FailFuture<ResolvedBytes> {
        self.try_resolve(address)
            .map_err(Err)
            .map_err(ready)
            .map_err(Box::pin)
            .unwrap_or_else(|x| x)
    }
}

impl<T> Origin for Ref<T> {
    type T = T;

    fn resolve(&self) -> FailFuture<Self::T> {
        self.origin.resolve()
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
    fn accept_refs(&self, visitor: &mut impl RefVisitor) {
        (**self).accept_refs(visitor);
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
