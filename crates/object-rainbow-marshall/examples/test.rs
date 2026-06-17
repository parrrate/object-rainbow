use object_rainbow_marshall::Marshalled;
use object_rainbow_point::IntoPoint;

fn main() -> anyhow::Result<()> {
    smol::block_on(async {
        Marshalled::new(().point()).await?;
        Ok(())
    })
}
