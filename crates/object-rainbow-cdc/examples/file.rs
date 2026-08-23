use object_rainbow_cdc::Chunks;

fn main() -> object_rainbow::Result<()> {
    smol::block_on(async move {
        let chunks = Chunks::from_seek(
            async || Ok(smol::fs::File::open("target/big.bin").await?),
            smol::unblock,
        )
        .await?;
        println!("{}", chunks.chunk_count());
        futures_util::io::copy(
            chunks.as_async_read(),
            &mut smol::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("target/big-out.bin")
                .await?,
        )
        .await?;
        Ok(())
    })
}
