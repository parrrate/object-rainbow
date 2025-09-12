use object_rainbow::{
    numeric::{Be, Le},
    *,
};

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

#[derive(Enum, ToOutput, Tagged, Topological, Parse, ParseInline, Size)]
#[enumtag("Le<u16>")]
enum Test<U, V, Y> {
    A,
    B(U),
    C { y: Y, x: V },
}

fn main() {
    println!("{}", hex::encode(DeriveExample::<(), ()>::HASH));
    println!("{}", DeriveExample::<Be<u8>, Le<u8>>::SIZE);
    println!("{}", Test::<(), (), ()>::SIZE);
}
