use std::{convert::Infallible, ops::Deref, sync::Arc};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, Hash, Input, Object, Parse, ParseInput,
    ParseSliceExtra, Point, PointInput, PointVisitor, RawPoint, Resolve, Singular, Tagged,
    ToOutput, ToOutputExt, Topological, length_prefixed::Lp,
};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct WithKey<K, Extra> {
    pub key: K,
    pub extra: Extra,
}

pub trait Key: 'static + Sized + Send + Sync + Clone {
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

type Resolution<K, Extra> =
    Arc<Lp<Vec<RawPoint<Encrypted<K, Infallible, Extra>, WithKey<K, Extra>>>>>;

#[derive(ToOutput, Clone)]
struct Unkeyed<T>(T);

impl<'a, T: Parse<Input<'a, Extra>>, K: 'static + Clone, Extra: 'static + Clone>
    Parse<Input<'a, WithKey<K, Extra>>> for Unkeyed<T>
{
    fn parse(input: Input<'a, WithKey<K, Extra>>) -> object_rainbow::Result<Self> {
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
    std::slice::Iter<'a, RawPoint<Encrypted<K, Infallible, Extra>, WithKey<K, Extra>>>;

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

    fn topology_hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        for point in self.resolution.iter() {
            hasher.update(point.hash());
        }
        hasher.finalize().into()
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
        let source = self.inner.output::<Vec<u8>>();
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
                .cast::<Encrypted<K, Vec<u8>, Extra>>()
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

impl<K: Key, T: for<'a> Parse<Input<'a, Extra>>, Extra: 'static + Send + Sync + Clone>
    Parse<Input<'_, WithKey<K, Extra>>> for Encrypted<K, T, Extra>
{
    fn parse(input: Input<'_, WithKey<K, Extra>>) -> object_rainbow::Result<Self> {
        let with_key = input.extra().clone();
        let resolve = input.resolve().clone();
        let source = with_key.key.decrypt(input.parse_all())?;
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
                        RawPoint<Encrypted<K, Infallible, Extra>, WithKey<K, Extra>>,
                        Error,
                    >,
                > + Send
                + 'static,
        >,
    >,
>;

struct ExtractResolution<'a, K, Extra>(&'a mut Extracted<K, Extra>, &'a K);

impl<K: Key, Extra: 'static + Send + Sync + Clone> PointVisitor<Extra>
    for ExtractResolution<'_, K, Extra>
{
    fn visit<T: Object<Extra>>(&mut self, point: &Point<T, Extra>) {
        let point = point.clone();
        let key = self.1.clone();
        self.0.push(Box::pin(async move {
            let point = encrypt_point(key, point).await?.raw().cast();
            Ok(point)
        }));
    }
}

pub async fn encrypt_point<K: Key, T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    key: K,
    point: Point<T, Extra>,
) -> object_rainbow::Result<Point<Encrypted<K, T, Extra>, WithKey<K, Extra>>> {
    if let Some((address, decrypt)) = point.extract_resolve::<Decrypt<K, Extra>>() {
        let point = decrypt
            .resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        return Ok(point.clone().cast().point());
    }
    let decrypted = point.fetch().await?;
    let encrypted = encrypt(key.clone(), decrypted).await?;
    let point = Point::from_object_extra(
        encrypted,
        WithKey {
            key,
            extra: point.extra().clone(),
        },
    );
    Ok(point)
}

pub async fn encrypt<K: Key, T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    key: K,
    decrypted: T,
) -> object_rainbow::Result<Encrypted<K, T, Extra>> {
    let mut futures = Vec::new();
    decrypted.accept_points(&mut ExtractResolution(&mut futures, &key));
    let resolution = futures_util::future::try_join_all(futures).await?;
    let resolution = Arc::new(Lp(resolution));
    let decrypted = Unkeyed(Arc::new(decrypted));
    let inner = EncryptedInner {
        resolution,
        decrypted,
    };
    Ok(Encrypted { key, inner })
}
