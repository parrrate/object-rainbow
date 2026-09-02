use std::time::Instant;

use futures_util::io::Cursor;
use object_rainbow::{FullHash, ToOutput};
use object_rainbow_cdc::{Chunk, Chunks, generate_tail};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

fn main() -> object_rainbow::Result<()> {
    {
        let rng = SmallRng::from_seed([216; _]);
        let data = rng.random_iter().take(1 << 30).collect::<Vec<_>>();
        let start = Instant::now();
        let chunks = smol::block_on(Chunks::in_memory(Cursor::new(data), smol::unblock))?;
        println!("{}", chunks.chunk_count());
        println!("{}s", start.elapsed().as_secs_f64());
        println!("{}", chunks.full_hash());
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
