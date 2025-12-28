use std::{collections::BTreeMap, io::Write};

use futures_util::future::try_join_all;
use object_rainbow::{
    Enum, Fetch, Inline, MaybeHasNiche, NicheForUnsized, Object, Output, Parse, ParseAsInline,
    ParseInline, ParseInput, Point, ReflessObject, SimpleObject, Size, SomeNiche, Tagged, ToOutput,
    Topological, ZeroNiche, length_prefixed::LpString, numeric::Le,
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
        serde_json::from_slice(input.parse_all()?)
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

#[derive(Enum, ToOutput, Topological, ParseAsInline, ParseInline, Clone)]
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
impl MaybeHasNiche for Distributed {
    type MnArray = NicheForUnsized;
}

impl Distributed {
    pub async fn to_value(&self) -> object_rainbow::Result<serde_json::Value> {
        Ok(match *self {
            Distributed::Null => serde_json::Value::Null,
            Distributed::Bool(x) => x.into(),
            Distributed::I64(x) => x.0.into(),
            Distributed::U64(x) => x.0.into(),
            Distributed::F64(x) => x.0.into(),
            Distributed::String(ref point) => point.fetch().await?.into(),
            Distributed::Array(ref point) => try_join_all(
                point
                    .fetch()
                    .await?
                    .into_iter()
                    .map(async |x| x.to_value().await),
            )
            .await?
            .into(),
            Distributed::Object(ref point) => try_join_all(
                point
                    .fetch()
                    .await?
                    .into_iter()
                    .map(async |(k, x)| Ok((k.0, x.to_value().await?))),
            )
            .await?
            .into_iter()
            .collect::<serde_json::Map<_, _>>()
            .into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DistributedParseError {
    #[error("invalid number")]
    InvalidNumber,
}

impl TryFrom<serde_json::Value> for Distributed {
    type Error = DistributedParseError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(x) => Self::Bool(x),
            serde_json::Value::Number(x) => {
                if let Some(x) = x.as_u64() {
                    Self::U64(x.into())
                } else if let Some(x) = x.as_i64() {
                    Self::I64(x.into())
                } else if let Some(x) = x.as_f64() {
                    Self::F64(x.into())
                } else {
                    return Err(DistributedParseError::InvalidNumber);
                }
            }
            serde_json::Value::String(x) => Self::String(x.point()),
            serde_json::Value::Array(vec) => Self::Array(
                vec.into_iter()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, _>>()?
                    .point(),
            ),
            serde_json::Value::Object(map) => Self::Object(
                map.into_iter()
                    .map(|(k, v)| Ok((LpString(k), Self::try_from(v)?)))
                    .collect::<Result<BTreeMap<_, _>, _>>()?
                    .point(),
            ),
        })
    }
}
