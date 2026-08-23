use object_rainbow_cdc::Chunks;

fn main() -> object_rainbow::Result<()> {
    smol::block_on(async move {
        let chunks = Chunks::from_file("target/big.bin").await?;
        println!("{}", chunks.chunk_count());
        chunks.to_file("target/big-out.bin").await?;
        Ok(())
    })
}
