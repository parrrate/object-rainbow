#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::sync::Arc;

use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    extra_option::ExtraOption, map_extra::TryMap,
};
#[cfg(feature = "point")]
use object_rainbow::{Fetch, FetchBytes, Singular};
#[cfg(feature = "point")]
use object_rainbow_point::Point;

#[cfg(feature = "point")]
use crate::IsMap;
use crate::{InlineValue, IsUnit, TailValue, ValueToA, dynamic::InlineDynamic};

#[derive(Enum, Debug, ToOutput, ListHashes, Topological, Parse, ParseInline, PartialEq)]
#[topology(unchecked)]
pub enum InlineMap {
    Point(
        #[cfg(feature = "point")] Point<Arc<Self>>,
        #[cfg(not(feature = "point"))] Infallible,
    ),
    I,
    K1(InlineDynamic),
    K,
    S2(Arc<Self>, Arc<Self>),
    S1(Arc<Self>),
    S,
}

impl InlineOutput for InlineMap {}

impl Tagged for InlineMap {}

impl TryMap<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> object_rainbow::Result<Self::Mapped> {
        Ok(match self {
            Self::Point(_) => {
                return Err(object_rainbow::error_operation!(
                    "point can only map in async"
                ));
            }
            Self::I => value,
            Self::K1(value) => value.value(),
            Self::K => Arc::new(InlineValue::Map(Arc::new(Self::K1(InlineDynamic::new(
                value,
            ))))),
            Self::S2(a, b) => a.map(value.clone())?.as_map()?.map(b.map(value)?)?,
            Self::S1(a) => Arc::new(InlineValue::Map(Arc::new(Self::S2(
                a.clone(),
                value.as_map()?,
            )))),
            Self::S => Arc::new(InlineValue::Map(Arc::new(Self::S1(value.as_map()?)))),
        })
    }
}

impl AsMap<Arc<InlineMap>> for InlineValue {
    fn as_map(&self) -> object_rainbow::Result<Arc<InlineMap>> {
        match self {
            Self::Option(ExtraOption::Some(value)) => value.as_map(),
            Self::Concat(a, b) if b.is_unit() => a.as_map(),
            Self::Concat(a, b) if a.is_unit() => b.as_map(),
            Self::Array(array)
                if let Some(first) = array.items.first()
                    && array.items.len() == 1
                    && let only = first =>
            {
                only.as_map()
            }
            Self::Enum(value) => value.value.as_map(),
            Self::Map(map) => Ok(map.clone()),
            #[cfg(feature = "point")]
            Self::Point(value) if value.extra.is_map() => Ok(Arc::new(InlineMap::Point(
                Point::from_singular(FetchInlineMap {
                    value: value.point.clone(),
                }),
            ))),
            _ => Err(object_rainbow::error_operation!("not a map")),
        }
    }
}

impl AsMap<Arc<InlineMap>> for TailValue {
    fn as_map(&self) -> object_rainbow::Result<Arc<InlineMap>> {
        match self {
            Self::Option(ExtraOption::Some(value)) => value.as_map(),
            Self::Concat(a, b) if b.is_unit() => a.as_map(),
            Self::Concat(a, b) if a.is_unit() => b.as_map(),
            Self::ToA(ValueToA(a, b)) if b.is_unit() => a.as_map(),
            Self::ToA(ValueToA(a, b)) if a.is_unit() => b.as_map(),
            Self::Enum(value) => value.value.as_map(),
            _ => Err(object_rainbow::error_operation!("not a map")),
        }
    }
}

pub trait AsMap<M> {
    fn as_map(&self) -> object_rainbow::Result<M>;
}

#[cfg(feature = "point")]
struct FetchInlineMap {
    value: Point<Arc<TailValue>>,
}

#[cfg(feature = "point")]
impl FetchBytes for FetchInlineMap {
    fn fetch_bytes(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::ByteNode> {
        self.value.fetch_bytes()
    }

    fn fetch_data(&'_ self) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        self.value.fetch_data()
    }
}

#[cfg(feature = "point")]
impl Fetch for FetchInlineMap {
    type T = Arc<InlineMap>;

    fn fetch_full(&'_ self) -> object_rainbow::FailFuture<'_, object_rainbow::Node<Self::T>> {
        Box::pin(async move {
            let (value, resolve) = self.value.fetch_full().await?;
            Ok((value.as_map()?, resolve))
        })
    }

    fn fetch(&'_ self) -> object_rainbow::FailFuture<'_, Self::T> {
        Box::pin(async move { self.value.fetch().await.and_then(|value| value.as_map()) })
    }
}

#[cfg(feature = "point")]
impl Singular for FetchInlineMap {
    fn hash(&self) -> object_rainbow::Hash {
        self.value.hash()
    }
}
