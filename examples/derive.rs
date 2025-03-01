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

#[derive(Enum, ToOutput, Topological, Parse, ParseInline)]
enum _Test<U, V, Y> {
    A,
    B(U),
    C { y: Y, x: V },
}
