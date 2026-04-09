use std::{fmt::Display, ops::Add};

use typenum::{Add1, B0, B1, ToInt, U0, U1};

use crate::*;

#[cfg(feature = "hex")]
mod hex;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    ParseAsInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Size,
)]
pub struct Hash([u8; HASH_SIZE]);

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0 {
            write!(f, "{x:X}")?;
        }
        Ok(())
    }
}

pub struct HashNiche<N>(N);

impl<N: ToInt<u8> + Add<B1>> Niche for HashNiche<N> {
    type NeedsTag = B0;
    type N = <Hash as Size>::Size;
    fn niche() -> GenericArray<u8, Self::N> {
        let mut niche = GenericArray::default();
        let last_byte = niche.len() - 1;
        niche[last_byte] = N::to_int();
        niche
    }
    type Next = SomeNiche<HashNiche<Add1<N>>>;
}

impl MaybeHasNiche for Hash {
    type MnArray = SomeNiche<HashNiche<U0>>;
}

impl<I: ParseInput> ParseInline<I> for Hash {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input
            .parse_inline::<OptionalHash>()?
            .get()
            .ok_or(Error::Zero)
    }
}

impl Hash {
    pub(crate) const fn from_sha256(hash: [u8; HASH_SIZE]) -> Self {
        Self(hash)
    }

    pub fn into_bytes(self) -> [u8; HASH_SIZE] {
        self.0
    }
}

impl Deref for Hash {
    type Target = [u8; HASH_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    ToOutput,
    InlineOutput,
    Parse,
    ParseInline,
    Tagged,
    ListHashes,
    Topological,
    Size,
    Default,
)]
pub struct OptionalHash([u8; HASH_SIZE]);

impl MaybeHasNiche for OptionalHash {
    type MnArray = SomeNiche<HashNiche<U1>>;
}

impl Equivalent<Option<Hash>> for OptionalHash {
    fn into_equivalent(self) -> Option<Hash> {
        self.get()
    }

    fn from_equivalent(object: Option<Hash>) -> Self {
        object.map(Self::from).unwrap_or_default()
    }
}

impl From<[u8; HASH_SIZE]> for OptionalHash {
    fn from(hash: [u8; HASH_SIZE]) -> Self {
        Self(hash)
    }
}

impl From<Hash> for OptionalHash {
    fn from(value: Hash) -> Self {
        value.0.into()
    }
}

impl OptionalHash {
    pub const NONE: Self = Self([0; HASH_SIZE]);

    pub fn get(&self) -> Option<Hash> {
        self.is_some().then_some(Hash(self.0))
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        *self == Self::NONE
    }

    pub fn unwrap(&self) -> Hash {
        self.get().unwrap()
    }

    pub fn clear(&mut self) {
        *self = Self::NONE;
    }
}

impl PartialEq<Hash> for OptionalHash {
    fn eq(&self, hash: &Hash) -> bool {
        self.0 == hash.0
    }
}

impl PartialEq<OptionalHash> for Hash {
    fn eq(&self, hash: &OptionalHash) -> bool {
        self.0 == hash.0
    }
}

#[test]
fn none_is_zeros() {
    assert_eq!(
        None::<Hash>.to_array().into_array(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    );
}

#[test]
fn none_none_is_one() {
    assert_eq!(
        None::<Option<Hash>>.to_array().into_array(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]
    );
}

#[test]
fn none_none_none_is_two() {
    assert_eq!(
        None::<Option<Option<Hash>>>.to_array().into_array(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ]
    );
}
