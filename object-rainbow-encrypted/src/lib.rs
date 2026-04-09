use std::{ops::Deref, sync::Arc};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, FetchBytes, Hash, ListHashes, Node, Object, Parse,
    ParseInline, ParseSliceExtra, PointInput, PointVisitor, Resolve, Singular, SingularFetch,
    Tagged, ToOutput, TopoVec, Topological, Traversible, derive_for_wrapped, length_prefixed::Lp,
    map_extra::MappedExtra, tuple_extra::Extra0,
};
use object_rainbow_point::{ExtractResolve, Extras, IntoPoint, Point};

#[derive_for_wrapped]
pub trait Key: 'static + Sized + Send + Sync + Clone + PartialEq + Eq {
    type Error: Into<anyhow::Error>;
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

#[derive(ToOutput, Parse, ParseInline, Clone)]
struct RawResolve<K> {
    key: Extras<K>,
    resolve: Arc<dyn Resolve>,
    addresses: Arc<Lp<Vec<Address>>>,
}

impl<K> RawResolve<K> {
    fn translate(&self, address: Address) -> object_rainbow::Result<Address> {
        self.addresses
            .get(address.index)
            .copied()
            .ok_or(Error::AddressOutOfBounds)
    }
}

fn side_parse<K: Key>(
    key: &K,
    data: &[u8],
    resolve: &Arc<dyn Resolve>,
) -> object_rainbow::Result<(InnerHeader<K>, Vec<u8>)> {
    let data = key
        .decrypt(data)
        .map_err(object_rainbow::Error::consistency)?;
    <(InnerHeader<K>, Vec<u8>) as ParseSliceExtra<K>>::parse_slice_extra(&data, resolve, key)
}

impl<K: Key> Resolve for RawResolve<K> {
    fn resolve<'a>(
        &'a self,
        address: Address,
        _: &'a Arc<dyn Resolve>,
    ) -> FailFuture<'a, ByteNode> {
        Box::pin(async move {
            let address = self.translate(address)?;
            let (data, resolve) = self.resolve.resolve(address, &self.resolve).await?;
            let (InnerHeader { resolve, .. }, data) = side_parse(&self.key.0, &data, &resolve)?;
            let resolve = Arc::new(resolve) as _;
            Ok((data, resolve))
        })
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let address = self.translate(address)?;
            let (data, resolve) = self.resolve.resolve(address, &self.resolve).await?;
            let (_, data) = side_parse(&self.key.0, &data, &resolve)?;
            Ok(data)
        })
    }

    fn try_resolve_local(
        &self,
        address: Address,
        _: &Arc<dyn Resolve>,
    ) -> object_rainbow::Result<Option<ByteNode>> {
        let address = self.translate(address)?;
        let Some((data, resolve)) = self.resolve.try_resolve_local(address, &self.resolve)? else {
            return Ok(None);
        };
        let (InnerHeader { resolve, .. }, data) = side_parse(&self.key.0, &data, &resolve)?;
        let resolve = Arc::new(resolve) as _;
        Ok(Some((data, resolve)))
    }
}

#[derive(Parse, ParseInline)]
struct InnerHeader<K> {
    tags: Hash,
    resolve: RawResolve<K>,
}

impl<K: Key> InnerHeader<K> {
    fn with<T: Topological + Tagged>(self, decrypted: T) -> object_rainbow::Result<Inner<K, T>> {
        if self.tags != T::HASH {
            return Err(object_rainbow::error_consistency!("tags mismatch"));
        }
        let mut topology = TopoVec::new();
        let mut v = RawVisit {
            at: 0,
            resolve: &self.resolve,
            visitor: &mut topology,
        };
        decrypted.traverse(&mut v);
        v.done()?;
        let topology = Arc::new(Lp(topology));
        let decrypted = Arc::new(decrypted);
        Ok(Inner {
            tags: self.tags,
            key: self.resolve.key,
            topology,
            decrypted,
        })
    }
}

#[derive(ToOutput)]
struct Inner<K, T> {
    tags: Hash,
    key: Extras<K>,
    topology: Arc<Lp<TopoVec>>,
    decrypted: Arc<T>,
}

struct RawVisit<'a, K, V> {
    at: usize,
    resolve: &'a RawResolve<K>,
    visitor: &'a mut V,
}

impl<'a, K, V> RawVisit<'a, K, V> {
    fn done(self) -> object_rainbow::Result<()> {
        if self.at != self.resolve.addresses.len() {
            Err(Error::AddressOutOfBounds)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct RawFetch<K, D> {
    key: K,
    resolve: Arc<dyn Resolve>,
    address: Address,
    decrypted: D,
}

impl<K, D> FetchBytes for RawFetch<K, D> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.resolve.resolve(self.address, &self.resolve)
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.resolve.resolve_data(self.address)
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.resolve.try_resolve_local(self.address, &self.resolve)
    }
}

impl<K: Key, D: Fetch<T: Topological + Tagged>> Fetch for RawFetch<K, D> {
    type T = Encrypted<K, D::T>;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move {
            let (encrypted, resolve) = self.resolve.resolve(self.address, &self.resolve).await?;
            let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
            let decrypted = self.decrypted.fetch().await?;
            let inner = header.with(decrypted)?;
            Ok((Encrypted { inner }, resolve))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move {
            let (encrypted, resolve) = self.resolve.resolve(self.address, &self.resolve).await?;
            let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
            let decrypted = self.decrypted.fetch().await?;
            let inner = header.with(decrypted)?;
            Ok(Encrypted { inner })
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((encrypted, resolve)) = self
            .resolve
            .try_resolve_local(self.address, &self.resolve)?
        else {
            return Ok(None);
        };
        let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
        let Some((decrypted, _)) = self.decrypted.try_fetch_local()? else {
            return Ok(None);
        };
        let inner = header.with(decrypted)?;
        Ok(Some((Encrypted { inner }, resolve)))
    }
}

impl<K: Send + Sync, D: Send + Sync> Singular for RawFetch<K, D> {
    fn hash(&self) -> Hash {
        self.address.hash
    }
}

impl<'a, K: Key, V: PointVisitor> PointVisitor for RawVisit<'a, K, V> {
    fn visit<T: Traversible>(&mut self, point: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let at = self.at;
        self.at += 1;
        if let Some(address) = self.resolve.addresses.get(at).copied() {
            let key = self.resolve.key.0.clone();
            let resolve = self.resolve.resolve.clone();
            let decrypted = point.clone();
            self.visitor.visit(&RawFetch {
                key,
                resolve,
                address,
                decrypted,
            });
        }
    }
}

impl<
    K: Key,
    T: Object<Extra>,
    Extra: 'static + Send + Sync + Clone,
    I: PointInput<Extra = (K, Extra)>,
> Parse<I> for Inner<K, T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let header = input
            .parse_inline::<MappedExtra<InnerHeader<K>, Extra0>>()?
            .1;
        let extra = input.extra().1.clone();
        let decrypted = T::parse_slice_extra(
            &input.parse_all()?,
            &(Arc::new(header.resolve.clone()) as _),
            &extra,
        )?;
        header.with(decrypted)
    }
}

impl<K: Clone, T> Clone for Inner<K, T> {
    fn clone(&self) -> Self {
        Self {
            tags: self.tags,
            key: self.key.clone(),
            topology: self.topology.clone(),
            decrypted: self.decrypted.clone(),
        }
    }
}

#[derive(Clone)]
struct InnerFetch<K, D> {
    key: K,
    encrypted: Arc<dyn Singular>,
    decrypted: D,
}

impl<K, D> FetchBytes for InnerFetch<K, D> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.encrypted.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.encrypted.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.encrypted.fetch_bytes_local()
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        self.encrypted.fetch_data_local()
    }
}

impl<K: Key, D: Fetch<T: Topological + Tagged>> Fetch for InnerFetch<K, D> {
    type T = Encrypted<K, D::T>;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move {
            let (encrypted, resolve) = self.encrypted.fetch_bytes().await?;
            let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
            let decrypted = self.decrypted.fetch().await?;
            let inner = header.with(decrypted)?;
            Ok((Encrypted { inner }, resolve))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move {
            let (encrypted, resolve) = self.encrypted.fetch_bytes().await?;
            let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
            let decrypted = self.decrypted.fetch().await?;
            let inner = header.with(decrypted)?;
            Ok(Encrypted { inner })
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((encrypted, resolve)) = self.encrypted.fetch_bytes_local()? else {
            return Ok(None);
        };
        let (header, _) = side_parse(&self.key, &encrypted, &resolve)?;
        let Some((decrypted, _)) = self.decrypted.try_fetch_local()? else {
            return Ok(None);
        };
        let inner = header.with(decrypted)?;
        Ok(Some((Encrypted { inner }, resolve)))
    }
}

impl<K: Send + Sync, D: Send + Sync> Singular for InnerFetch<K, D> {
    fn hash(&self) -> Hash {
        self.encrypted.hash()
    }
}

struct IterateResolution<'a, 'r, K, V> {
    key: &'a K,
    topology: &'r mut std::slice::Iter<'a, Arc<dyn Singular>>,
    visitor: &'a mut V,
}

impl<'a, K: Key, V: PointVisitor> PointVisitor for IterateResolution<'a, '_, K, V> {
    fn visit<T: Traversible>(&mut self, decrypted: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let decrypted = decrypted.clone();
        let encrypted = self.topology.next().expect("length mismatch").clone();
        let point = Point::from_fetch(
            encrypted.hash(),
            InnerFetch {
                key: self.key.clone(),
                decrypted,
                encrypted,
            }
            .into_dyn_fetch(),
        );
        self.visitor.visit(&point);
    }
}

impl<K, T> ListHashes for Inner<K, T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.topology.list_hashes(f);
    }

    fn topology_hash(&self) -> Hash {
        self.topology.0.data_hash()
    }

    fn point_count(&self) -> usize {
        self.topology.len()
    }
}

impl<K: Key, T: Topological> Topological for Inner<K, T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        let topology = &mut self.topology.iter();
        self.decrypted.traverse(&mut IterateResolution {
            key: &self.key.0,
            topology,
            visitor,
        });
        assert!(topology.next().is_none());
    }

    fn topology(&self) -> TopoVec {
        self.topology.0.clone()
    }
}

pub struct Encrypted<K, T> {
    inner: Inner<K, T>,
}

impl<K, T: Clone> Encrypted<K, T> {
    pub fn into_inner(self) -> T {
        Arc::unwrap_or_clone(self.inner.decrypted)
    }
}

impl<K, T> Deref for Encrypted<K, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.decrypted.as_ref()
    }
}

impl<K: Clone, T> Clone for Encrypted<K, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K, T> ListHashes for Encrypted<K, T> {
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

impl<K: Key, T: Topological> Topological for Encrypted<K, T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        self.inner.traverse(visitor);
    }
}

impl<K: Key, T: ToOutput> ToOutput for Encrypted<K, T> {
    fn to_output(&self, output: &mut dyn object_rainbow::Output) {
        if output.is_mangling() {
            output.write(
                &self
                    .inner
                    .key
                    .encrypt(b"this encrypted constant is followed by an unencrypted inner hash"),
            );
            self.inner.decrypted.data_hash();
        }
        if output.is_real() {
            let source = self.inner.vec();
            output.write(&self.inner.key.encrypt(&source));
        }
    }
}

trait EncryptedExtra<K>: 'static + Send + Sync + Clone {
    type Extra: 'static + Send + Sync + Clone;
    fn parts(&self) -> (K, Self::Extra);
}

impl<K: 'static + Send + Sync + Clone, Extra: 'static + Send + Sync + Clone> EncryptedExtra<K>
    for (K, Extra)
{
    type Extra = Extra;

    fn parts(&self) -> (K, Self::Extra) {
        self.clone()
    }
}

impl<K: 'static + Send + Sync + Clone> EncryptedExtra<K> for K {
    type Extra = ();

    fn parts(&self) -> (K, Self::Extra) {
        (self.clone(), ())
    }
}

impl<
    K: Key,
    T: Object<Extra>,
    Extra: 'static + Send + Sync + Clone,
    I: PointInput<Extra: EncryptedExtra<K, Extra = Extra>>,
> Parse<I> for Encrypted<K, T>
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let with_key = input.extra().parts();
        let resolve = input.resolve().clone();
        let source = with_key
            .0
            .decrypt(&input.parse_all()?)
            .map_err(object_rainbow::Error::consistency)?;
        let inner = Inner::<K, T>::parse_slice_extra(&source, &resolve, &with_key)?;
        Ok(Self { inner })
    }
}

impl<K, T> Tagged for Encrypted<K, T> {}

type Extracted =
    Vec<std::pin::Pin<Box<dyn Future<Output = Result<Arc<dyn Singular>, Error>> + Send + 'static>>>;

struct ExtractResolution<'a, K> {
    extracted: &'a mut Extracted,
    key: &'a K,
}

impl<K: Key> PointVisitor for ExtractResolution<'_, K> {
    fn visit<T: Traversible>(&mut self, decrypted: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let decrypted = decrypted.clone();
        let key = self.key.clone();
        self.extracted.push(Box::pin(async move {
            Ok(Arc::new(encrypt_point(key, decrypted).await?) as _)
        }));
    }
}

pub async fn encrypt_point<K: Key, T: Traversible>(
    key: K,
    decrypted: impl 'static + SingularFetch<T = T>,
) -> object_rainbow::Result<Point<Encrypted<K, T>>> {
    if let Some((address, resolve)) = decrypted.extract_resolve::<RawResolve<K>>()
        && resolve.key.0 == key
    {
        let address = resolve.translate(*address)?;
        let point = Point::from_fetch(
            address.hash,
            RawFetch {
                key,
                resolve: resolve.resolve.clone(),
                address,
                decrypted,
            }
            .into_dyn_fetch(),
        );
        return Ok(point);
    }
    let decrypted = decrypted.fetch().await?;
    let encrypted = encrypt(key, decrypted).await?;
    let point = encrypted.point();
    Ok(point)
}

pub async fn encrypt<K: Key, T: Traversible>(
    key: K,
    decrypted: T,
) -> object_rainbow::Result<Encrypted<K, T>> {
    let mut futures = Vec::with_capacity(decrypted.point_count());
    decrypted.traverse(&mut ExtractResolution {
        extracted: &mut futures,
        key: &key,
    });
    let topology = futures_util::future::try_join_all(futures).await?;
    let topology = Arc::new(Lp(topology));
    let decrypted = Arc::new(decrypted);
    let inner = Inner {
        tags: T::HASH,
        key: Extras(key),
        topology,
        decrypted,
    };
    Ok(Encrypted { inner })
}
