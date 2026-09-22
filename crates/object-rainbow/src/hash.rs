use std::{fmt::Display, ops::Add};

use typenum::{Add1, B0, B1, ToInt, U0, U16, U32};

use crate::*;

#[cfg(feature = "hex")]
mod hex;

/// Valid [`Hash`]. Has restrictions on its byte layout (e.g. cannot be all zeroes);
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

impl Default for Hash {
    fn default() -> Self {
        "".data_hash()
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0 {
            write!(f, "{x:02X}")?;
        }
        Ok(())
    }
}

pub struct HashNiche<N>(N);

impl<N: ToInt<u8> + Add<B1>> Niche for HashNiche<N> {
    type NeedsTag = B0;
    type Cut = B0;
    type N = <Hash as Size>::Size;
    fn niche() -> GenericArray<u8, Self::N> {
        let mut niche: GenericArray<u8, U32> = GenericArray::default();
        let last = u128::MAX - u128::from(N::to_int());
        let second: &mut GenericArray<u8, U16> =
            <&mut GenericArray<u8, U32> as Split<u8, U16>>::split(&mut niche).1;
        *second = GenericArray::<u8, U16>::from(last.to_be_bytes());
        niche
    }
    type Next = SomeNiche<HashNiche<Add1<N>>>;
}

impl MaybeHasNiche for Hash {
    type MnArray = SomeNiche<HashNiche<U0>>;
}

impl<N: ToInt<u8> + Add<B1>> MinNiche for HashNiche<N> {}

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

    /// Convert into raw bytes.
    pub fn into_bytes(self) -> [u8; HASH_SIZE] {
        self.0
    }
}

impl FromOutput for Hash {
    type Output = sha2::Sha256;
}

impl From<sha2::Sha256> for Hash {
    fn from(hasher: sha2::Sha256) -> Self {
        Self::from_sha256(hasher.finalize().into())
    }
}

impl From<Hash> for [u8; HASH_SIZE] {
    fn from(hash: Hash) -> Self {
        hash.into_bytes()
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

/// Some level of `Option<Hash>` but more explicitly represented as `[u8; HASH_SIZE]`.
#[pod]
pub struct OptionalHash([u8; HASH_SIZE]);

impl Display for OptionalHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(hash) = self.get() {
            write!(f, "{hash}")?;
        } else {
            write!(f, "NONE")?;
        }
        Ok(())
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
    /// No [`Hash`].
    pub const NONE: Self = Self([0; HASH_SIZE]);

    /// Get [`Hash`] if it's valid.
    pub fn get(&self) -> Option<Hash> {
        self.is_some().then_some(Hash(self.0))
    }

    /// Check whether this is a [`Hash`].
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Check whether this is [`Self::NONE`] or some less-nested [`None`].
    pub fn is_none(&self) -> bool {
        self.0[..16] == Self::NONE.0[..16]
    }

    /// Get [`Hash`] or panic.
    pub fn unwrap(&self) -> Hash {
        self.get().unwrap()
    }

    /// Set to [`Self::NONE`].
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

impl ByteOrd for Hash {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[test]
fn none_is_zeros() {
    assert_eq!(
        &*None::<Hash>.to_array(),
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ]
    );
}

#[test]
fn none_none_is_one() {
    assert_eq!(
        &*None::<Option<Hash>>.to_array(),
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        ]
    );
}

#[test]
fn none_none_none_is_two() {
    assert_eq!(
        &*None::<Option<Option<Hash>>>.to_array(),
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd,
        ]
    );
}
