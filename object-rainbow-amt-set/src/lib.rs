use object_rainbow::{
    Hash, Inline, InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged,
    ToOutput, Topological, assert_impl,
};
use object_rainbow_hamt::HamtMap;

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Size,
    MaybeHasNiche,
    Clone,
    Default,
    PartialEq,
    Eq,
)]
pub struct AmtSet(HamtMap<()>);

assert_impl!(
    impl<E> Inline<E> for AmtSet where E: 'static + Send + Sync + Clone {}
);

impl AmtSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&mut self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.insert(hash, ()).await?.is_none())
    }

    pub async fn contains(&self, hash: Hash) -> object_rainbow::Result<bool> {
        Ok(self.0.get(hash).await?.is_some())
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use object_rainbow::ToOutput;
    use smol_macros::test;

    use crate::AmtSet;

    #[apply(test!)]
    async fn test() -> object_rainbow::Result<()> {
        let mut set = AmtSet::default();
        assert!(set.insert(1u8.data_hash()).await?);
        assert!(set.contains(1u8.data_hash()).await?);
        assert!(!set.insert(1u8.data_hash()).await?);
        assert!(set.contains(1u8.data_hash()).await?);
        assert!(set.insert(2u8.data_hash()).await?);
        assert!(set.contains(1u8.data_hash()).await?);
        assert!(set.contains(2u8.data_hash()).await?);
        assert!(!set.insert(2u8.data_hash()).await?);
        assert!(set.contains(1u8.data_hash()).await?);
        assert!(set.contains(2u8.data_hash()).await?);
        Ok(())
    }
}
