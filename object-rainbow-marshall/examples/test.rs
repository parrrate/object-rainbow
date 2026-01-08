use object_rainbow::Traversible;
use object_rainbow_marshall::Marshalled;

fn main() -> anyhow::Result<()> {
    smol::block_on(async {
        Marshalled::new(().point()).await?;
        Ok(())
    })
}
