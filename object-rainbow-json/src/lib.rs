use std::io::Write;

use object_rainbow::{
    MaybeHasNiche, Object, Output, Parse, ParseInput, ReflessObject, Size, SomeNiche, Tagged,
    ToOutput, Topological, ZeroNiche,
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
