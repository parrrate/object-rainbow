use object_rainbow::ToOutput;
use object_rainbow_cdc::{Chunk, generate_tail};

fn main() -> object_rainbow::Result<()> {
    {
        let original = b"325074";
        let chunk = Chunk::new(original)?;
        let data = smol::block_on(chunk.fetch())?;
        assert_eq!(original.as_slice(), data.as_slice());
    }
    {
        let original = &vec![0u8; 1 << 24];
        let chunk = Chunk::new(original)?;
        let data = smol::block_on(chunk.fetch())?;
        assert_eq!(original.as_slice(), data.as_slice());
    }
    for n in 0..1024 {
        let n = generate_tail(&(n, vec![0u8; 1 << 24]).vec())?;
        println!("{n:?}");
    }
    Ok(())
}
