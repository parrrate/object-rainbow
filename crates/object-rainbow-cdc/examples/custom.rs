use std::time::Instant;

use futures_util::{
    TryStreamExt,
    io::{BufReader, Cursor},
};
use object_rainbow_cdc::fastcdc::ChunkStream;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

fn main() -> std::io::Result<()> {
    smol::block_on(async move {
        let rng = SmallRng::from_seed([216; _]);
        let data = rng.random_iter().take(1 << 30).collect::<Vec<_>>();
        for _ in 0..10 {
            let start = Instant::now();
            let chunks = ChunkStream::new(BufReader::with_capacity(1 << 16, Cursor::new(&data)))
                .map_ok(|_| {})
                .try_collect::<Vec<_>>()
                .await?;
            println!("{} {}s", chunks.len(), start.elapsed().as_secs_f64());
        }
        Ok(())
    })
}
