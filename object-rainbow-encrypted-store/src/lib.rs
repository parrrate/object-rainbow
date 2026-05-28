use object_rainbow::{DiffHashes, ObjectHashes, ToOutput};
use object_rainbow_encrypted::Key;
use object_rainbow_store::ExternalStore;

#[derive(Clone, PartialEq)]
pub struct EncryptedStore<S, K> {
    store: S,
    key: K,
}

impl<S, K> EncryptedStore<S, K> {
    pub fn new(store: S, key: K) -> Self {
        Self { store, key }
    }
}

impl<S: ExternalStore, K: Key> ExternalStore for EncryptedStore<S, K> {
    type Id = S::Id;

    async fn save_data(
        &self,
        data: &[u8],
        refs: &[Self::Id],
        hashes: ObjectHashes,
    ) -> object_rainbow::Result<Self::Id> {
        let encrypted = &self.key.encrypt(data);
        let diff = DiffHashes {
            tags: Default::default(),
            topology: refs.data_hash(),
            mangle: (self.key.mangle_prefix().data_hash(), hashes.data).data_hash(),
        };
        self.store
            .save_data(
                encrypted,
                refs,
                ObjectHashes {
                    diff: diff.data_hash(),
                    data: encrypted.data_hash(),
                },
            )
            .await
    }

    async fn contains_data(
        &self,
        data: &[u8],
        refs: &[Self::Id],
        hashes: ObjectHashes,
    ) -> object_rainbow::Result<bool> {
        let encrypted = &self.key.encrypt(data);
        let diff = DiffHashes {
            tags: Default::default(),
            topology: refs.data_hash(),
            mangle: (self.key.mangle_prefix().data_hash(), hashes.data).data_hash(),
        };
        self.store
            .contains_data(
                encrypted,
                refs,
                ObjectHashes {
                    diff: diff.data_hash(),
                    data: encrypted.data_hash(),
                },
            )
            .await
    }

    async fn contains(&self, id: &Self::Id) -> object_rainbow::Result<bool> {
        self.store.contains(id).await
    }

    #[expect(refining_impl_trait)]
    async fn fetch(&self, id: &Self::Id) -> object_rainbow::Result<Vec<u8>> {
        self.key
            .decrypt(self.store.fetch(id).await?.as_ref())
            .map_err(object_rainbow::Error::consistency)
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use macro_rules_attribute::apply;
    use object_rainbow::{FullHash, Hash, ObjectHashes, ToOutput};
    use object_rainbow_encrypted::{Key, encrypt};
    use object_rainbow_point::IntoPoint;
    use object_rainbow_store::ExternalStore;
    use smol_macros::test;

    use crate::EncryptedStore;

    #[derive(Clone, PartialEq)]
    struct NormalStore;

    impl ExternalStore for NormalStore {
        type Id = Hash;

        async fn save_data(
            &self,
            _: &[u8],
            _: &[Self::Id],
            hashes: ObjectHashes,
        ) -> object_rainbow::Result<Self::Id> {
            Ok(hashes.data_hash())
        }

        async fn contains_data(
            &self,
            _: &[u8],
            _: &[Self::Id],
            _: ObjectHashes,
        ) -> object_rainbow::Result<bool> {
            unimplemented!()
        }

        async fn contains(&self, _: &Self::Id) -> object_rainbow::Result<bool> {
            unimplemented!()
        }

        #[expect(refining_impl_trait)]
        async fn fetch(&self, _: &Self::Id) -> object_rainbow::Result<Vec<u8>> {
            unimplemented!()
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct InverseKey;

    impl Key for InverseKey {
        type Error = Infallible;

        fn encrypt(&self, data: &[u8]) -> Vec<u8> {
            data.iter().copied().map(|x| !x).collect()
        }

        fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
            Ok(data.iter().copied().map(|x| !x).collect())
        }
    }

    #[apply(test!)]
    async fn equivalent_to_normal() -> object_rainbow::Result<()> {
        let o = ((*b"a"), ((*b"b"), (*b"c").point()).point());
        let e = encrypt(InverseKey, o.clone()).await?;
        let f = e.full_hash();
        let s = EncryptedStore::new(NormalStore, InverseKey);
        let h = s.store_object(o).await?;
        assert_eq!(f, h);
        Ok(())
    }
}
