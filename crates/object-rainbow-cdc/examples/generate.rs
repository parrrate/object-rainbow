use std::time::Instant;

use futures_util::io::Cursor;
use object_rainbow::ToOutput;
use object_rainbow_cdc::{Chunk, Chunks, generate_tail};
use rand::random_iter;

fn main() -> object_rainbow::Result<()> {
    {
        let data = random_iter().take(1 << 30).collect::<Vec<_>>();
        let start = Instant::now();
        let chunks = smol::block_on(Chunks::new(Cursor::new(data), smol::unblock))?;
        println!("{}", chunks.chunk_len());
        println!("{}s", start.elapsed().as_secs());
    }
    {
        let original = b"325074";
        let chunk = Chunk::new(original)?;
        let data = smol::block_on(chunk.data())?;
        assert_eq!(original.as_slice(), data.as_slice());
    }
    {
        let original = &vec![0u8; 1 << 24];
        let chunk = Chunk::new(original)?;
        let data = smol::block_on(chunk.data())?;
        assert_eq!(original.as_slice(), data.as_slice());
    }
    for n in 0..1024 {
        let n = generate_tail(&(n, vec![0u8; 1 << 24]).vec())?;
        println!("{n:?}");
    }
    Ok(())
}
