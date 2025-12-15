use std::{convert::Infallible, ops::Deref, sync::Arc};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, Hash, Input, Object, Parse, ParseInput,
    ParseSlice, ParseSliceExtra, Point, PointInput, PointVisitor, RawPoint, Resolve, Singular,
    Tagged, ToOutput, ToOutputExt, Topological, length_prefixed::Lp,
};
use sha2::{Digest, Sha256};

pub trait Key: 'static + Sized + Send + Sync + Clone {
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

type Resolution<K> = Arc<Lp<Vec<RawPoint<Encrypted<K, Infallible>, K>>>>;

#[derive(ToOutput, Clone)]
struct Unkeyed<T>(T);

impl<'a, T: Parse<Input<'a>>, K> Parse<Input<'a, K>> for Unkeyed<T> {
    fn parse(input: Input<'a, K>) -> object_rainbow::Result<Self> {
        Ok(Self(T::parse(input.replace_extra(&()))?))
    }
}

#[derive(ToOutput, Parse)]
struct EncryptedInner<K, T> {
    resolution: Resolution<K>,
    decrypted: Unkeyed<Arc<T>>,
}

impl<K, T> Clone for EncryptedInner<K, T> {
    fn clone(&self) -> Self {
        Self {
            resolution: self.resolution.clone(),
            decrypted: self.decrypted.clone(),
        }
    }
}

struct IterateResolution<'a, K, V> {
    resolution: std::slice::Iter<'a, RawPoint<Encrypted<K, Infallible>, K>>,
    visitor: &'a mut V,
}

impl<'a, K: Key, V: PointVisitor<K>> PointVisitor for IterateResolution<'a, K, V> {
    fn visit<T: Object>(&mut self, _: &Point<T>) {
        let point = self
            .resolution
            .next()
            .expect("length mismatch")
            .clone()
            .cast::<Encrypted<K, T>>()
            .point();
        self.visitor.visit(&point);
    }
}

impl<K: Key, T: Topological> Topological<K> for EncryptedInner<K, T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<K>) {
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

pub struct Encrypted<K, T> {
    key: K,
    inner: EncryptedInner<K, T>,
}

impl<K, T: Clone> Encrypted<K, T> {
    pub fn into_inner(self) -> T {
        Arc::unwrap_or_clone(self.inner.decrypted.0)
    }
}

impl<K, T> Deref for Encrypted<K, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.decrypted.0.as_ref()
    }
}

impl<K: Clone, T> Clone for Encrypted<K, T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<K: Key, T: Topological> Topological<K> for Encrypted<K, T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor<K>) {
        self.inner.accept_points(visitor);
    }

    fn topology_hash(&self) -> Hash {
        self.inner.topology_hash()
    }
}

impl<K: Key, T: ToOutput> ToOutput for Encrypted<K, T> {
    fn to_output(&self, output: &mut dyn object_rainbow::Output) {
        let source = self.inner.output::<Vec<u8>>();
        output.write(&self.key.encrypt(&source));
    }
}

#[derive(Clone)]
struct Decrypt<K> {
    resolution: Resolution<K>,
}

impl<K: Key> Resolve for Decrypt<K> {
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
                .cast::<Encrypted<K, Vec<u8>>>()
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

impl<K: Key, T: for<'a> Parse<Input<'a>>> Parse<Input<'_, K>> for Encrypted<K, T> {
    fn parse(input: Input<'_, K>) -> object_rainbow::Result<Self> {
        let key = input.extra().clone();
        let resolve = input.resolve().clone();
        let source = key.decrypt(input.parse_all())?;
        let EncryptedInner {
            resolution,
            decrypted,
        } = EncryptedInner::<K, Vec<u8>>::parse_slice_extra(&source, &resolve, &key)?;
        let decrypted = T::parse_slice(
            &decrypted.0,
            &(Arc::new(Decrypt {
                resolution: resolution.clone(),
            }) as _),
        )?;
        let decrypted = Unkeyed(Arc::new(decrypted));
        let inner = EncryptedInner {
            resolution,
            decrypted,
        };
        Ok(Self { key, inner })
    }
}

impl<K, T> Tagged for Encrypted<K, T> {}

impl<K: Key, T: Object> Object<K> for Encrypted<K, T> {}

struct ExtractResolution<'a, K>(
    &'a mut Vec<FailFuture<'static, RawPoint<Encrypted<K, Infallible>, K>>>,
    &'a K,
);

impl<K: Key> PointVisitor for ExtractResolution<'_, K> {
    fn visit<T: Object>(&mut self, point: &Point<T>) {
        let point = point.clone();
        let key = self.1.clone();
        self.0.push(Box::pin(async move {
            let point = encrypt_point(key, point).await?.raw().cast();
            Ok(point)
        }));
    }
}

pub async fn encrypt_point<K: Key, T: Object>(
    key: K,
    point: Point<T>,
) -> object_rainbow::Result<Point<Encrypted<K, T>, K>> {
    if let Some((address, decrypt)) = point.extract_resolve::<Decrypt<K>>() {
        let point = decrypt
            .resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?;
        return Ok(point.clone().cast().point());
    }
    let decrypted = point.fetch().await?;
    let encrypted = encrypt(key.clone(), decrypted).await?;
    let point = Point::from_object_extra(encrypted, key);
    Ok(point)
}

pub async fn encrypt<K: Key, T: Object>(
    key: K,
    decrypted: T,
) -> object_rainbow::Result<Encrypted<K, T>> {
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
