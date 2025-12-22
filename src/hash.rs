use crate::*;

#[derive(
    Debug,
    ToOutput,
    Tagged,
    Topological,
    ParseAsInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Object,
    Inline,
    ReflessObject,
    ReflessInline,
)]
pub struct Hash([u8; HASH_SIZE]);

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
    ParseAsInline,
    ParseInline,
    Tagged,
    Topological,
    Object,
    Inline,
    ReflessObject,
    ReflessInline,
)]
pub struct OptionalHash([u8; HASH_SIZE]);

impl Default for OptionalHash {
    fn default() -> Self {
        Self::NONE
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

    pub(crate) fn unwrap(&self) -> Hash {
        self.get().unwrap()
    }

    pub(crate) fn clear(&mut self) {
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
