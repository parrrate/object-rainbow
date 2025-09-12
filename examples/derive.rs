use object_rainbow::{numeric::Le, *};

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

#[derive(Enum, ToOutput)]
enum _Test {
    A,
    B(Le<i32>),
    C { x: Le<i32> },
}
