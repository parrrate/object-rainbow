use object_rainbow_fetchall::fetchall;
use object_rainbow_point::IntoPoint;

fn main() -> anyhow::Result<()> {
    smol::block_on(async move {
        let map = fetchall(&(().point(), (().point(), ().point()).point())).await?;
        println!("{}", map.len());
        Ok(())
    })
}
