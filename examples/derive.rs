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
    MaybeHasNiche,
)]
#[tags("example")]
pub struct DeriveExample<A, B> {
    a: A,
    #[tags(skip)]
    b: B,
}

#[derive(
    Enum,
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
    MaybeHasNiche,
)]
#[enumtag("Le<u16>")]
enum Test<U, V, Y> {
    A,
    B(U),
    C { y: Y, x: V },
}

#[derive(
    Enum,
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
    MaybeHasNiche,
)]
#[enumtag("Le<u16>")]
enum Stuff<T> {
    A(T),
    B(T),
    C(T),
    D(T),
    E(T),
}

fn main() {
    println!("{}", hex::encode(DeriveExample::<(), ()>::HASH));
    println!("{}", DeriveExample::<Be<u8>, Le<u8>>::SIZE);
    println!("{}", Test::<(), (), ()>::SIZE);
    println!("{}", Option::<Test<(), (), ()>>::SIZE);
    println!("{}", Option::<Stuff<bool>>::SIZE);
    println!("{:?}", None::<Stuff<(bool, ())>>.output::<Vec<u8>>());
}
