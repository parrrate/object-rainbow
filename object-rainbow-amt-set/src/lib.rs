pub use object_rainbow_hamt::HamtSet as AmtSet;

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
