//! serialized objects
//!
//! ```txt
//! | length | data | index | index | ... |
//! | length | data | index | index | ... |
//! | length | data | index | index | ... |
//! ...
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
    sync::Arc,
};

use object_rainbow::{
    Address, ByteNode, FailFuture, Fetch, FetchBytes, Hash, ListHashes, Node, Object, Output,
    Parse, ParseInput, ParseSliceExtra, PointInput, Resolve, Singular, Tagged, ToOutput,
    Topological, Traversible,
};
use object_rainbow_fetchall::fetchall;
use object_rainbow_local_map::LocalMap;

#[derive(Clone)]
struct MarshalledInner {
    data: Arc<[u8]>,
    root: Hash,
    at: usize,
}

impl MarshalledInner {
    fn read_usize(&self, at: usize) -> object_rainbow::Result<usize> {
        u64::from_le_bytes(
            self.data
                .get(
                    at..at
                        .checked_add(8)
                        .ok_or(object_rainbow::Error::UnsupportedLength)?,
                )
                .ok_or(object_rainbow::Error::UnsupportedLength)?
                .try_into()
                .unwrap(),
        )
        .try_into()
        .map_err(|_| object_rainbow::Error::UnsupportedLength)
    }

    fn data_len(&self) -> object_rainbow::Result<usize> {
        self.read_usize(self.at)
    }

    fn data_begin(&self) -> object_rainbow::Result<usize> {
        self.at
            .checked_add(8)
            .ok_or(object_rainbow::Error::UnsupportedLength)
    }

    fn data(&self) -> object_rainbow::Result<&[u8]> {
        Ok(&self.data[self.data_begin()?..self.data_end()?])
    }

    fn data_end(&self) -> object_rainbow::Result<usize> {
        self.data_begin()?
            .checked_add(self.data_len()?)
            .ok_or(object_rainbow::Error::UnsupportedLength)
    }

    fn reference_at(&self, index: usize) -> object_rainbow::Result<usize> {
        self.read_usize(
            self.data_end()?
                .checked_add(
                    index
                        .checked_mul(8)
                        .ok_or(object_rainbow::Error::UnsupportedLength)?,
                )
                .ok_or(object_rainbow::Error::UnsupportedLength)?,
        )
    }

    fn data_vec(&self) -> object_rainbow::Result<Vec<u8>> {
        Ok(self.data()?.into())
    }

    fn resolve_node(&self, address: Address) -> object_rainbow::Result<ByteNode> {
        let referenced = MarshalledInner {
            data: self.data.clone(),
            root: address.hash,
            at: self.reference_at(address.index)?,
        };
        Ok((referenced.data_vec()?, referenced.to_resolve()))
    }

    fn to_resolve(&self) -> Arc<dyn Resolve> {
        Arc::new(self.clone())
    }
}

impl FetchBytes for MarshalledInner {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        Box::pin(async move { Ok((self.data_vec()?, self.to_resolve())) })
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move { self.data_vec() })
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        Ok(Some((self.data_vec()?, self.to_resolve())))
    }
}

impl Resolve for MarshalledInner {
    fn resolve(&'_ self, address: Address) -> FailFuture<'_, ByteNode> {
        Box::pin(async move { self.resolve_node(address) })
    }

    fn resolve_data(&'_ self, address: Address) -> FailFuture<'_, Vec<u8>> {
        Box::pin(async move {
            let (data, _) = self.resolve_node(address)?;
            Ok(data)
        })
    }

    fn try_resolve_local(&self, address: Address) -> object_rainbow::Result<Option<ByteNode>> {
        self.resolve_node(address).map(Some)
    }
}

impl Singular for MarshalledInner {
    fn hash(&self) -> Hash {
        self.root
    }
}

enum Action {
    WriteLocation { at: usize, of: Hash },
    SaveFull { hash: Hash },
    FinishLocation { at: usize, of: Hash },
}

trait ToBytes: Copy {
    fn to_bytes(self) -> [u8; 8];
}

impl ToBytes for u64 {
    fn to_bytes(self) -> [u8; 8] {
        self.to_le_bytes()
    }
}

impl ToBytes for usize {
    fn to_bytes(self) -> [u8; 8] {
        (self as u64).to_bytes()
    }
}

#[derive(Clone)]
pub struct MarshalledRoot {
    marshalled: MarshalledInner,
}

impl FetchBytes for MarshalledRoot {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.marshalled.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.marshalled.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.marshalled.fetch_bytes_local()
    }
}

impl Singular for MarshalledRoot {
    fn hash(&self) -> Hash {
        self.marshalled.hash()
    }
}

pub fn marshall(map: &LocalMap, root: Hash) -> MarshalledRoot {
    let mut data = Vec::<u8>::new();
    let mut locations = BTreeMap::<Hash, usize>::new();
    let mut started = BTreeSet::<Hash>::new();
    let mut stack = vec![Action::SaveFull { hash: root }];
    while let Some(action) = stack.pop() {
        match action {
            Action::WriteLocation { at, of } => {
                data[at..at + 8].copy_from_slice(&locations.get(&of).unwrap().to_bytes());
            }
            Action::SaveFull { hash } => {
                if locations.contains_key(&hash) {
                    continue;
                }
                assert!(started.insert(hash));
                let (references, d) = map.get(hash).unwrap();
                stack.push(Action::FinishLocation {
                    at: data.len(),
                    of: hash,
                });
                data.extend_from_slice(&d.len().to_bytes());
                data.extend_from_slice(d);
                for &hash in references {
                    stack.push(Action::WriteLocation {
                        at: data.len(),
                        of: hash,
                    });
                    data.extend_from_slice(&u64::MAX.to_bytes());
                    stack.push(Action::SaveFull { hash });
                }
            }
            Action::FinishLocation { at, of } => {
                assert!(started.contains(&of));
                assert!(locations.insert(of, at).is_none());
            }
        }
    }
    assert_eq!(*locations.get(&root).unwrap(), 0);
    let data = Arc::from(data);
    let marshalled = MarshalledInner { data, root, at: 0 };
    MarshalledRoot { marshalled }
}

impl ToOutput for MarshalledRoot {
    fn to_output(&self, output: &mut dyn Output) {
        self.marshalled.root.to_output(output);
        self.marshalled.data.to_output(output);
    }
}

impl Tagged for MarshalledRoot {}
impl ListHashes for MarshalledRoot {}
impl Topological for MarshalledRoot {}

impl<I: ParseInput> Parse<I> for MarshalledRoot {
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let root = input.parse_inline()?;
        let data = Arc::<[u8]>::from(input.parse_all()?.as_ref());
        let marshalled = MarshalledInner { data, root, at: 0 };
        Ok(Self { marshalled })
    }
}

#[derive(Tagged)]
pub struct Marshalled<T> {
    root: MarshalledRoot,
    object: T,
}

impl<T> ToOutput for Marshalled<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.root.to_output(output);
    }
}

impl<T> ListHashes for Marshalled<T> {}
impl<T> Topological for Marshalled<T> {}

impl<I: PointInput, T: Object<I::Extra>> Parse<I> for Marshalled<T> {
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let extra = input.extra().clone();
        let root = input.parse::<MarshalledRoot>()?;
        let object = T::parse_slice_extra(
            root.marshalled.data()?,
            &root.marshalled.to_resolve(),
            &extra,
        )?;
        if object.full_hash() != root.hash() {
            return Err(object_rainbow::Error::FullHashMismatch);
        }
        Ok(Self { root, object })
    }
}

impl<T: ToOutput> FetchBytes for Marshalled<T> {
    fn fetch_bytes(&'_ self) -> FailFuture<'_, ByteNode> {
        self.root.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> FailFuture<'_, Vec<u8>> {
        self.root.fetch_data()
    }

    fn fetch_bytes_local(&self) -> object_rainbow::Result<Option<ByteNode>> {
        self.root.fetch_bytes_local()
    }

    fn fetch_data_local(&self) -> Option<Vec<u8>> {
        Some(self.object.output())
    }
}

impl<T: Send + Sync + Clone + ToOutput> Fetch for Marshalled<T> {
    type T = T;

    fn fetch_full(&'_ self) -> FailFuture<'_, Node<Self::T>> {
        Box::pin(async move { Ok((self.object.clone(), self.root.marshalled.to_resolve())) })
    }

    fn fetch(&'_ self) -> FailFuture<'_, Self::T> {
        Box::pin(async move { Ok(self.object.clone()) })
    }

    fn try_fetch_local(&self) -> object_rainbow::Result<Option<Node<Self::T>>> {
        Ok(Some((
            self.object.clone(),
            self.root.marshalled.to_resolve(),
        )))
    }

    fn fetch_local(&self) -> Option<Self::T> {
        Some(self.object.clone())
    }

    fn try_unwrap(self: Arc<Self>) -> Option<Self::T> {
        Arc::try_unwrap(self).ok().map(|Self { object, .. }| object)
    }
}

impl<T: Send + Sync + ToOutput> Singular for Marshalled<T> {
    fn hash(&self) -> Hash {
        self.root.hash()
    }
}

impl<T: Traversible> Marshalled<T> {
    pub async fn new(object: T) -> object_rainbow::Result<Self> {
        let map = fetchall(&object).await?;
        let root = marshall(&map, object.full_hash());
        Ok(Self { root, object })
    }
}

impl<T> Deref for Marshalled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
