use std::collections::BTreeMap;

use futures_util::future::try_join_all;
use object_rainbow::{
    Enum, Fetch, InlineOutput, ListHashes, MaybeHasNiche, NicheForUnsized, NoNiche, Parse,
    ParseInline, Tagged, ToOutput, Topological, length_prefixed::LpString, numeric::Le,
};
use object_rainbow_point::{IntoPoint, Point};
use serde::{Deserialize, Serialize};

#[derive(
    Enum,
    ToOutput,
    InlineOutput,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Default,
    Serialize,
    Deserialize,
)]
#[serde(untagged)]
#[topology(recursive)]
pub enum Distributed {
    #[default]
    Null,
    Bool(bool),
    I64(Le<i64>),
    U64(Le<u64>),
    F64(Le<f64>),
    String(Point<String>),
    Array(#[parse(unchecked)] Point<Vec<Self>>),
    Object(#[parse(unchecked)] Point<BTreeMap<LpString, Self>>),
}

impl Tagged for Distributed {}
impl MaybeHasNiche for Distributed {
    type MnArray = NoNiche<NicheForUnsized>;
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
            Distributed::Object(ref point) => {
                try_join_all(
                    point.fetch().await?.into_iter().map(async |(k, x)| {
                        Ok::<_, object_rainbow::Error>((k.0, x.to_value().await?))
                    }),
                )
                .await?
                .into_iter()
                .collect::<serde_json::Map<_, _>>()
                .into()
            }
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
