use object_rainbow::ToOutput;
use object_rainbow_cdc::generate_tail;

fn main() {
    for n in 0..1024 {
        let n = generate_tail(&(n, vec![0u8; 1 << 24]).vec());
        println!("{n}");
    }
}
