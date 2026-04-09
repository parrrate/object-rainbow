use object_rainbow::{Fetch, FullHash, Traversible};
use object_rainbow_store::RainbowStore;
use object_rainbow_store_opendal::OpendalStore;
use opendal::{Operator, services::Memory};

fn main() -> anyhow::Result<()> {
    smol::block_on(async move {
        let store = OpendalStore::from_operator(Operator::new(Memory::default())?.finish());
        let mut point = store
            .saved_point(
                &((*b"alisa", *b"feistel").point(), [1, 2, 3, 4].point()).point(),
                (),
            )
            .await?;
        assert_eq!(point.fetch().await?.0.fetch().await?.0, *b"alisa");
        assert_eq!(point.fetch().await?.0.fetch().await?.1, *b"feistel");
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 4]);
        println!("{}", hex::encode(point.full_hash()));
        point.fetch_mut().await?.1.fetch_mut().await?[3] = 5;
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 5]);
        println!("{}", hex::encode(point.full_hash()));
        point.fetch_mut().await?.1.fetch_mut().await?[3] = 4;
        assert_eq!(point.fetch().await?.1.fetch().await?, [1, 2, 3, 4]);
        println!("{}", hex::encode(point.full_hash()));
        Ok(())
    })
}
