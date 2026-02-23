use object_rainbow_history_store::HistoryStore;
use object_rainbow_store_opendal::OpendalStore;
use object_rainbow_trie::TrieMap;
use opendal::{Operator, services::Memory};

fn main() -> anyhow::Result<()> {
    smol::block_on(async {
        let store = HistoryStore::<TrieMap<Vec<u8>, u8>, (Option<u8>, Vec<u8>), _>::new(
            "main",
            OpendalStore::from_operator(Operator::new(Memory::default())?.finish()),
        );
        store.commit((Some(123), b"abc".into())).await?;
        assert_eq!(store.load().await?.get(&b"abc".into()).await?.unwrap(), 123);
        store.commit((None, b"abc".into())).await?;
        assert!(store.load().await?.get(&b"abc".into()).await?.is_none());
        Ok(())
    })
}
