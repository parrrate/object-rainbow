use object_rainbow::Traversible;
use object_rainbow_fetchall::fetchall;

fn main() -> anyhow::Result<()> {
    smol::block_on(async move {
        let map = fetchall(&(().point(), (().point(), ().point()).point())).await?;
        println!("{}", map.len());
        Ok(())
    })
}
