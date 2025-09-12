use std::{
    any::{Any, TypeId},
    convert::Infallible,
    ops::Deref,
    sync::Arc,
};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, FetchBytes, Hash, Input, Object, Parse,
    ParseInput, ParseSlice, Point, PointVisitor, RawPoint, Resolve, Singular, Tagged, ToOutput,
    ToOutputExt, Topological, length_prefixed::Lp,
};
use sha2::{Digest, Sha256};

pub trait Key: 'static + Sized + Send + Sync + Clone {
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

type Resolution<K> = Arc<Lp<Vec<RawPoint<Encrypted<K, Infallible>>>>>;

#[derive(ToOutput, Parse)]
struct EncryptedInner<K, T> {
    resolution: Resolution<K>,
    decrypted: Arc<T>,
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
    resolution: std::slice::Iter<'a, RawPoint<Encrypted<K, Infallible>>>,
    visitor: &'a mut V,
}

impl<'a, K: Key, V: PointVisitor> PointVisitor for IterateResolution<'a, K, V> {
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

impl<K: Key, T: Topological> Topological for EncryptedInner<K, T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.decrypted.accept_points(&mut IterateResolution {
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
            key: self.key.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<K: Key, T: Topological> Topological for Encrypted<K, T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
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
    key: K,
    resolution: Resolution<K>,
}

impl<K: Key> Resolve for Decrypt<K> {
    fn resolve(&self, address: Address) -> FailFuture<ByteNode> {
        Box::pin(async move {
            let Encrypted {
                key,
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
                Arc::into_inner(decrypted).expect("not shared because reconstructed"),
                Arc::new(Decrypt { key, resolution }) as _,
            ))
        })
    }

    fn resolve_extension(
        &self,
        address: Address,
        typeid: TypeId,
    ) -> object_rainbow::Result<&dyn Any> {
        self.resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?
            .extension(typeid)
    }

    fn extension(&self, typeid: TypeId) -> object_rainbow::Result<&dyn Any> {
        if typeid == TypeId::of::<K>() {
            Ok(&self.key)
        } else {
            Err(Error::UnknownExtension)
        }
    }

    fn name(&self) -> &str {
        "decrypt"
    }
}

impl<K: Key, T: for<'a> Parse<Input<'a>>> Parse<Input<'_>> for Encrypted<K, T> {
    fn parse(input: Input<'_>) -> object_rainbow::Result<Self> {
        let key = input.extension::<K>()?.clone();
        let resolve = input.resolve().clone();
        let source = key.decrypt(input.parse_all())?;
        let EncryptedInner {
            resolution,
            decrypted,
        } = EncryptedInner::<K, Vec<u8>>::parse_slice(&source, &resolve)?;
        let decrypted = T::parse_slice(
            &decrypted,
            &(Arc::new(Decrypt {
                key: key.clone(),
                resolution: resolution.clone(),
            }) as _),
        )?;
        let decrypted = Arc::new(decrypted);
        let inner = EncryptedInner {
            resolution,
            decrypted,
        };
        Ok(Self { key, inner })
    }
}

impl<K, T> Tagged for Encrypted<K, T> {}

impl<K: Key, T: Object> Object for Encrypted<K, T> {
    fn extension(&self, typeid: TypeId) -> object_rainbow::Result<&dyn Any> {
        if typeid == TypeId::of::<K>() {
            Ok(&self.key)
        } else {
            Err(Error::UnknownExtension)
        }
    }
}

struct ExtractResolution<'a, K>(
    &'a mut Vec<FailFuture<'static, RawPoint<Encrypted<K, Infallible>>>>,
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
) -> object_rainbow::Result<Point<Encrypted<K, T>>> {
    if let Some((address, decrypt)) = point.extract_resolve::<Decrypt<K>>() {
        let point = decrypt.resolution.get(address.index).ok_or(Error::AddressOutOfBounds)?;
        return Ok(point.clone().cast().point());
    }
    let decrypted = point.fetch().await?;
    let encrypted = encrypt(key, decrypted).await?;
    let point = Point::from_object(encrypted);
    Ok(point)
}

pub async fn encrypt<K: Key, T: Object>(
    key: K,
    decrypted: T,
) -> object_rainbow::Result<Encrypted<K, T>> {
    let mut futures = Vec::new();
    decrypted.accept_points(&mut ExtractResolution(&mut futures, &key));
    let mut resolution = Vec::new();
    for future in futures {
        resolution.push(future.await?);
    }
    let resolution = Arc::new(Lp(resolution));
    let decrypted = Arc::new(decrypted);
    let inner = EncryptedInner {
        resolution,
        decrypted,
    };
    Ok(Encrypted { key, inner })
}
