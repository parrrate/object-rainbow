use std::{ops::Deref, sync::Arc};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, Hash, Object, Parse, ParseSliceExtra, Point,
    PointInput, PointVisitor, RawPoint, Resolve, Tagged, ToOutput, Topological,
    length_prefixed::Lp,
};

#[derive(Clone)]
pub struct WithKey<K, Extra> {
    pub key: K,
    pub extra: Extra,
}

pub trait Key: 'static + Sized + Send + Sync + Clone {
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

type Resolution<K, Extra> = Arc<Lp<Vec<RawPoint<Encrypted<K, Vec<u8>, Extra>, WithKey<K, Extra>>>>>;

#[derive(ToOutput, Clone)]
struct Unkeyed<T>(T);

impl<
    T: Parse<I::WithExtra<Extra>>,
    K: 'static + Clone,
    Extra: 'static + Clone,
    I: PointInput<Extra = WithKey<K, Extra>>,
> Parse<I> for Unkeyed<T>
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        Ok(Self(T::parse(
            input.map_extra(|WithKey { extra, .. }| extra),
        )?))
    }
}

#[derive(ToOutput, Parse)]
struct EncryptedInner<K, T, Extra> {
    resolution: Resolution<K, Extra>,
    decrypted: Unkeyed<Arc<T>>,
}

impl<K, T, Extra> Clone for EncryptedInner<K, T, Extra> {
    fn clone(&self) -> Self {
        Self {
            resolution: self.resolution.clone(),
            decrypted: self.decrypted.clone(),
        }
    }
}

type ResolutionIter<'a, K, Extra> =
    std::slice::Iter<'a, RawPoint<Encrypted<K, Vec<u8>, Extra>, WithKey<K, Extra>>>;

struct IterateResolution<'a, K, V, Extra> {
    resolution: ResolutionIter<'a, K, Extra>,
    visitor: &'a mut V,
}

impl<'a, K: Key, V: PointVisitor<WithKey<K, Extra>>, Extra: 'static + Send + Sync + Clone>
    PointVisitor<Extra> for IterateResolution<'a, K, V, Extra>
{
    fn visit<T: Object<Extra>>(&mut self, _: &Point<T, Extra>) {
        let point = self
            .resolution
            .next()
            .expect("length mismatch")
            .clone()
            .cast::<Encrypted<K, T, Extra>>()
            .point();
        self.visitor.visit(&point);
    }
}

impl<K: Key, T: Topological<Extra>, Extra: 'static + Send + Sync + Clone>
    Topological<WithKey<K, Extra>> for EncryptedInner<K, T, Extra>
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<WithKey<K, Extra>>) {
        self.decrypted.0.accept_points(&mut IterateResolution {
            resolution: self.resolution.iter(),
            visitor,
        });
    }

    fn point_count(&self) -> usize {
        self.resolution.len()
    }

    fn topology_hash(&self) -> Hash {
        self.resolution.0.data_hash()
    }
}

pub struct Encrypted<K, T, Extra> {
    key: K,
    inner: EncryptedInner<K, T, Extra>,
}

impl<K, T: Clone, Extra> Encrypted<K, T, Extra> {
    pub fn into_inner(self) -> T {
        Arc::unwrap_or_clone(self.inner.decrypted.0)
    }
}

impl<K, T, Extra> Deref for Encrypted<K, T, Extra> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.decrypted.0.as_ref()
    }
}

impl<K: Clone, T, Extra> Clone for Encrypted<K, T, Extra> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<K: Key, T: Topological<Extra>, Extra: 'static + Send + Sync + Clone>
    Topological<WithKey<K, Extra>> for Encrypted<K, T, Extra>
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<WithKey<K, Extra>>) {
        self.inner.accept_points(visitor);
    }

    fn topology_hash(&self) -> Hash {
        self.inner.topology_hash()
    }
}

impl<K: Key, T: ToOutput, Extra> ToOutput for Encrypted<K, T, Extra> {
    fn to_output(&self, output: &mut dyn object_rainbow::Output) {
        let source = self.inner.vec();
        output.write(&self.key.encrypt(&source));
    }
}

#[derive(Clone)]
struct Decrypt<K, Extra> {
    resolution: Resolution<K, Extra>,
}

impl<K: Key, Extra: 'static + Send + Sync + Clone> Resolve for Decrypt<K, Extra> {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        Box::pin(async move {
            let Encrypted {
                key: _,
                inner:
                    EncryptedInner {
                        resolution,
                        decrypted,
                    },
            } = self
                .resolution
                .get(address.index)
                .ok_or(Error::AddressOutOfBounds)?
                .clone()
                .fetch()
                .await?;
            Ok((
                Arc::into_inner(decrypted.0).expect("not shared because reconstructed"),
                Arc::new(Decrypt { resolution }) as _,
            ))
        })
    }

    fn name(&self) -> &str {
        "decrypt"
    }
}

impl<
    K: Key,
    T: Object<Extra>,
    Extra: 'static + Send + Sync + Clone,
    I: PointInput<Extra = WithKey<K, Extra>>,
> Parse<I> for Encrypted<K, T, Extra>
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let with_key = input.extra().clone();
        let resolve = input.resolve().clone();
        let source = with_key.key.decrypt(input.parse_all()?)?;
        let EncryptedInner {
            resolution,
            decrypted,
        } = EncryptedInner::<K, Vec<u8>, Extra>::parse_slice_extra(&source, &resolve, &with_key)?;
        let decrypted = T::parse_slice_extra(
            &decrypted.0,
            &(Arc::new(Decrypt {
                resolution: resolution.clone(),
            }) as _),
            &with_key.extra,
        )?;
        let decrypted = Unkeyed(Arc::new(decrypted));
        let inner = EncryptedInner {
            resolution,
            decrypted,
        };
        Ok(Self {
            key: with_key.key,
            inner,
        })
    }
}

impl<K, T, Extra> Tagged for Encrypted<K, T, Extra> {}

impl<K: Key, T: Object<Extra>, Extra: 'static + Send + Sync + Clone> Object<WithKey<K, Extra>>
    for Encrypted<K, T, Extra>
{
}

type Extracted<K, Extra> = Vec<
    std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        RawPoint<Encrypted<K, Vec<u8>, Extra>, WithKey<K, Extra>>,
                        Error,
                    >,
                > + Send
                + 'static,
        >,
    >,
>;

struct ExtractResolution<'a, K, Extra> {
    extracted: &'a mut Extracted<K, Extra>,
    key: &'a K,
    extra: &'a Extra,
}

impl<K: Key, Extra: 'static + Send + Sync + Clone> PointVisitor<Extra>
    for ExtractResolution<'_, K, Extra>
{
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        let point = point.clone();
        let key = self.key.clone();
        let extra = self.extra.clone();
        self.extracted.push(Box::pin(async move {
            let point = encrypt_point(key.clone(), point, &extra)
                .await?
                .raw(WithKey { key, extra })
                .cast();
            Ok(point)
        }));
    }
}

pub async fn encrypt_point<K: Key, T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    key: K,
    point: Point<T, Extra>,
    extra: &Extra,
) -> object_rainbow::Result<Point<Encrypted<K, T, Extra>, WithKey<K, Extra>>> {
    if let Some((address, decrypt)) = point.extract_resolve::<Decrypt<K, Extra>>() {
        let point = decrypt
            .resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        return Ok(point.clone().cast().point());
    }
    let decrypted = point.fetch().await?;
    let encrypted = encrypt(key.clone(), decrypted, extra).await?;
    let point = encrypted.point_extra();
    Ok(point)
}

pub async fn encrypt<K: Key, T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    key: K,
    decrypted: T,
    extra: &Extra,
) -> object_rainbow::Result<Encrypted<K, T, Extra>> {
    let mut futures = Vec::with_capacity(decrypted.point_count());
    decrypted.accept_points(&mut ExtractResolution {
        extracted: &mut futures,
        key: &key,
        extra,
    });
    let resolution = futures_util::future::try_join_all(futures).await?;
    let resolution = Arc::new(Lp(resolution));
    let decrypted = Unkeyed(Arc::new(decrypted));
    let inner = EncryptedInner {
        resolution,
        decrypted,
    };
    Ok(Encrypted { key, inner })
}
