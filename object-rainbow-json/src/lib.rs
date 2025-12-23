use std::{collections::BTreeMap, io::Write};

use object_rainbow::{
    Enum, Inline, MaybeHasNiche, Object, Output, Parse, ParseAsInline, ParseInline, ParseInput,
    Point, ReflessObject, Size, SomeNiche, Tagged, ToOutput, Topological, ZeroNiche,
    length_prefixed::LpString, numeric::Le,
};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Json<T>(pub T);

struct Writer<'a> {
    output: &'a mut dyn Output,
}

impl Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T: Serialize> ToOutput for Json<T> {
    fn to_output(&self, output: &mut dyn Output) {
        serde_json::to_writer(&mut Writer { output }, &self.0)
            .expect("json write errors are considered bugs");
    }
}

impl<T: DeserializeOwned, I: ParseInput> Parse<I> for Json<T> {
    fn parse(input: I) -> object_rainbow::Result<Self> {
        serde_json::from_slice(input.parse_all())
            .map_err(object_rainbow::Error::parse)
            .map(Self)
    }
}

impl<T> Topological for Json<T> {}
impl<T> Tagged for Json<T> {}
impl<T: 'static + Send + Sync + Serialize + DeserializeOwned> Object for Json<T> {}
impl<T: 'static + Send + Sync + Serialize + DeserializeOwned> ReflessObject for Json<T> {}

impl Size for Json<()> {
    type Size = object_rainbow::typenum::consts::U4;
}

impl MaybeHasNiche for Json<()> {
    type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
}

#[derive(Enum, ToOutput, Topological, ParseAsInline, ParseInline, Size, MaybeHasNiche)]
pub enum Distributed {
    Null,
    Bool(bool),
    I64(Le<i64>),
    U64(Le<u64>),
    F64(Le<f64>),
    String(Point<String>),
    Array(Point<Vec<Self>>),
    Object(Point<BTreeMap<LpString, Self>>),
}

impl Tagged for Distributed {}
impl Object for Distributed {}
impl Inline for Distributed {}
