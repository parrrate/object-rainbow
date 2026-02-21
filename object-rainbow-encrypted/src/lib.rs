use std::{ops::Deref, sync::Arc};

use object_rainbow::{
    Address, ByteNode, Error, FailFuture, Fetch, FetchBytes, FullHash, Hash, ListHashes, Node,
    Object, Parse, ParseSliceExtra, PointInput, PointVisitor, Resolve, Singular, SingularFetch,
    Tagged, ToOutput, Topological, Traversible, length_prefixed::Lp,
};
use object_rainbow_point::{ExtractResolve, IntoPoint, Point};

pub trait Key: 'static + Sized + Send + Sync + Clone {
    fn encrypt(&self, data: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

type Resolution<K> = Arc<Lp<Vec<Point<Encrypted<K, Vec<u8>>>>>>;

#[derive(ToOutput, Clone)]
struct Unkeyed<T>(T);

impl<
    T: Parse<I::WithExtra<Extra>>,
    K: 'static + Clone,
    Extra: 'static + Clone,
    I: PointInput<Extra = (K, Extra)>,
> Parse<I> for Unkeyed<T>
{
    fn parse(input: I) -> object_rainbow::Result<Self> {
        Ok(Self(T::parse(input.map_extra(|(_, extra)| extra))?))
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

type ResolutionIter<'a, K> = std::slice::Iter<'a, Point<Encrypted<K, Vec<u8>>>>;

struct IterateResolution<'a, 'r, K, V> {
    resolution: &'r mut ResolutionIter<'a, K>,
    visitor: &'a mut V,
}

struct Visited<K, P> {
    decrypted: P,
    encrypted: Point<Encrypted<K, Vec<u8>>>,
}

impl<K, P> FetchBytes for Visited<K, P> {
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

impl<K, P: Send + Sync> Singular for Visited<K, P> {
    fn hash(&self) -> Hash {
        self.encrypted.hash()
    }
}

impl<K: Key, P: Fetch<T: Traversible>> Fetch for Visited<K, P> {
    type T = Encrypted<K, P::T>;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move {
            let (
                Encrypted {
                    key,
                    inner:
                        EncryptedInner {
                            resolution,
                            decrypted: _,
                        },
                },
                resolve,
            ) = self.encrypted.fetch_full().await?;
            let decrypted = self.decrypted.fetch().await?;
            let decrypted = Unkeyed(Arc::new(decrypted));
            Ok((
                Encrypted {
                    key,
                    inner: EncryptedInner {
                        resolution,
                        decrypted,
                    },
                },
                resolve,
            ))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move {
            let Encrypted {
                key,
                inner:
                    EncryptedInner {
                        resolution,
                        decrypted: _,
                    },
            } = self.encrypted.fetch().await?;
            let decrypted = self.decrypted.fetch().await?;
            let decrypted = Unkeyed(Arc::new(decrypted));
            Ok(Encrypted {
                key,
                inner: EncryptedInner {
                    resolution,
                    decrypted,
                },
            })
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((
            Encrypted {
                key,
                inner:
                    EncryptedInner {
                        resolution,
                        decrypted: _,
                    },
            },
            resolve,
        )) = self.encrypted.try_fetch_local()?
        else {
            return Ok(None);
        };
        let Some((decrypted, _)) = self.decrypted.try_fetch_local()? else {
            return Ok(None);
        };
        let decrypted = Unkeyed(Arc::new(decrypted));
        Ok(Some((
            Encrypted {
                key,
                inner: EncryptedInner {
                    resolution,
                    decrypted,
                },
            },
            resolve,
        )))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        let Encrypted {
            key,
            inner:
                EncryptedInner {
                    resolution,
                    decrypted: _,
                },
        } = self.encrypted.fetch_local()?;
        let decrypted = Unkeyed(Arc::new(self.decrypted.fetch_local()?));
        Some(Encrypted {
            key,
            inner: EncryptedInner {
                resolution,
                decrypted,
            },
        })
    }
}

impl<'a, K: Key, V: PointVisitor> PointVisitor for IterateResolution<'a, '_, K, V> {
    fn visit<T: Traversible>(&mut self, decrypted: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let decrypted = decrypted.clone();
        let encrypted = self.resolution.next().expect("length mismatch").clone();
        let point = Point::from_fetch(
            encrypted.hash(),
            Arc::new(Visited {
                decrypted,
                encrypted,
            }),
        );
        self.visitor.visit(&point);
    }
}

impl<K, T> ListHashes for EncryptedInner<K, T> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.resolution.list_hashes(f);
    }

    fn topology_hash(&self) -> Hash {
        self.resolution.0.data_hash()
    }

    fn point_count(&self) -> usize {
        self.resolution.len()
    }
}

impl<K: Key, T: Topological> Topological for EncryptedInner<K, T> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        let resolution = &mut self.resolution.iter();
        self.decrypted.0.traverse(&mut IterateResolution {
            resolution,
            visitor,
        });
        assert!(resolution.next().is_none());
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
        let source = self.inner.vec();
        output.write(&self.key.encrypt(&source));
    }
}

#[derive(Clone)]
struct Decrypt<K> {
    resolution: Resolution<K>,
}

impl<K: Key> Decrypt<K> {
    async fn resolve_bytes(
        &self,
        address: Address,
    ) -> object_rainbow::Result<(Vec<u8>, Resolution<K>)> {
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
        let data = Arc::unwrap_or_clone(decrypted.0);
        Ok((data, resolution))
    }
}

impl<K: Key> Resolve for Decrypt<K> {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        Box::pin(async move {
            let (data, resolution) = self.resolve_bytes(address).await?;
            Ok((data, Arc::new(Decrypt { resolution }) as _))
        })
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let (data, _) = self.resolve_bytes(address).await?;
            Ok(data)
        })
    }

    fn try_resolve_local(&self, address: Address) -> object_rainbow::Result<Option<ByteNode>> {
        let Some((
            Encrypted {
                key: _,
                inner:
                    EncryptedInner {
                        resolution,
                        decrypted,
                    },
            },
            _,
        )) = self
            .resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?
            .clone()
            .try_fetch_local()?
        else {
            return Ok(None);
        };
        let data = Arc::unwrap_or_clone(decrypted.0);
        Ok(Some((data, Arc::new(Decrypt { resolution }) as _)))
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
        let source = with_key.0.decrypt(&input.parse_all()?)?;
        let EncryptedInner {
            resolution,
            decrypted,
        } = EncryptedInner::<K, Vec<u8>>::parse_slice_extra(&source, &resolve, &with_key)?;
        let decrypted = T::parse_slice_extra(
            &decrypted.0,
            &(Arc::new(Decrypt {
                resolution: resolution.clone(),
            }) as _),
            &with_key.1,
        )?;
        let decrypted = Unkeyed(Arc::new(decrypted));
        let inner = EncryptedInner {
            resolution,
            decrypted,
        };
        Ok(Self {
            key: with_key.0,
            inner,
        })
    }
}

impl<K, T> Tagged for Encrypted<K, T> {}

type Extracted<K> = Vec<
    std::pin::Pin<
        Box<dyn Future<Output = Result<Point<Encrypted<K, Vec<u8>>>, Error>> + Send + 'static>,
    >,
>;

struct ExtractResolution<'a, K> {
    extracted: &'a mut Extracted<K>,
    key: &'a K,
}

struct Untyped<K, T> {
    key: (K, ()),
    encrypted: Point<Encrypted<K, T>>,
}

impl<K, T> FetchBytes for Untyped<K, T> {
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

impl<K: Send + Sync, T> Singular for Untyped<K, T> {
    fn hash(&self) -> Hash {
        self.encrypted.hash()
    }
}

impl<K: Key, T: FullHash> Fetch for Untyped<K, T> {
    type T = Encrypted<K, Vec<u8>>;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move {
            let (data, resolve) = self.fetch_bytes().await?;
            let encrypted = Self::T::parse_slice_extra(&data, &resolve, &self.key)?;
            Ok((encrypted, resolve))
        })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move {
            let (data, resolve) = self.fetch_bytes().await?;
            let encrypted = Self::T::parse_slice_extra(&data, &resolve, &self.key)?;
            Ok(encrypted)
        })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        let Some((data, resolve)) = self.fetch_bytes_local()? else {
            return Ok(None);
        };
        let encrypted = Self::T::parse_slice_extra(&data, &resolve, &self.key)?;
        Ok(Some((encrypted, resolve)))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        let Encrypted {
            key,
            inner:
                EncryptedInner {
                    resolution,
                    decrypted,
                },
        } = self.encrypted.fetch_local()?;
        let decrypted = Unkeyed(Arc::new(decrypted.vec()));
        Some(Encrypted {
            key,
            inner: EncryptedInner {
                resolution,
                decrypted,
            },
        })
    }
}

impl<K: Key> PointVisitor for ExtractResolution<'_, K> {
    fn visit<T: Traversible>(&mut self, decrypted: &(impl 'static + SingularFetch<T = T> + Clone)) {
        let decrypted = decrypted.clone();
        let key = self.key.clone();
        self.extracted.push(Box::pin(async move {
            let encrypted = encrypt_point(key.clone(), decrypted).await?;
            let encrypted = Point::from_fetch(
                encrypted.hash(),
                Arc::new(Untyped {
                    key: (key, ()),
                    encrypted,
                }),
            );
            Ok(encrypted)
        }));
    }
}

pub async fn encrypt_point<K: Key, T: Traversible>(
    key: K,
    decrypted: impl 'static + SingularFetch<T = T>,
) -> object_rainbow::Result<Point<Encrypted<K, T>>> {
    if let Some((address, decrypt)) = decrypted.extract_resolve::<Decrypt<K>>() {
        let encrypted = decrypt
            .resolution
            .get(address.index)
            .ok_or(Error::AddressOutOfBounds)?
            .clone();
        let point = Point::from_fetch(
            encrypted.hash(),
            Arc::new(Visited {
                decrypted,
                encrypted,
            }),
        );
        return Ok(point);
    }
    let decrypted = decrypted.fetch().await?;
    let encrypted = encrypt(key.clone(), decrypted).await?;
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
    let resolution = futures_util::future::try_join_all(futures).await?;
    let resolution = Arc::new(Lp(resolution));
    let decrypted = Unkeyed(Arc::new(decrypted));
    let inner = EncryptedInner {
        resolution,
        decrypted,
    };
    Ok(Encrypted { key, inner })
}
