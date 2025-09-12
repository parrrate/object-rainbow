use object_rainbow::*;

#[derive(
    ToOutput,
    Topological,
    Tagged,
    Object,
    Inline,
    ReflessObject,
    ReflessInline,
    Size,
    Parse,
    ParseInline,
)]
#[tags("example")]
pub struct DeriveExample<A, B> {
    a: A,
    #[tags(skip)]
    b: B,
}

fn main() {
    println!("{}", hex::encode(DeriveExample::<(), ()>::HASH));
}
