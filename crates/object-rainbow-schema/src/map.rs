#[cfg(not(feature = "point"))]
use std::convert::Infallible;
use std::{collections::BTreeSet, sync::Arc};

use futures_util::future::try_join;
use object_rainbow::{
    Enum, InlineOutput, ListHashes, Parse, ParseInline, Tagged, ToOutput, Topological,
    extra_option::ExtraOption, map_extra::TryMap, u63::U63,
};
#[cfg(feature = "point")]
use object_rainbow::{Fetch, FetchBytes, Singular};
#[cfg(feature = "apply")]
use object_rainbow_apply::Apply;
#[cfg(feature = "point")]
use object_rainbow_point::Point;

#[cfg(feature = "point")]
use crate::IsMap;
use crate::{AbstractCollection, InlineValue, IsUnit, TailValue, ValueToA, dynamic::InlineDynamic};

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
    Index(U63),
    Unpack,
    Pack1(InlineDynamic),
    Pack,
}

impl InlineOutput for InlineMap {}

impl Tagged for InlineMap {}

impl TryMap<Arc<InlineValue>> for InlineMap {
    type Mapped = Arc<InlineValue>;

    fn map(&self, value: Arc<InlineValue>) -> object_rainbow::Result<Self::Mapped> {
        Ok(match self {
            Self::Point(_) => {
                return Err(object_rainbow::error_operation!(
                    "point can only map in async",
                ));
            }
            Self::I => value,
            Self::K1(value) => value.value(),
            Self::K => Arc::new(Self::K1(InlineDynamic::new(value)).into()),
            Self::S2(a, b) => a.map(value.clone())?.as_map()?.map(b.map(value)?)?,
            Self::S1(a) => Arc::new(Self::S2(a.clone(), value.as_map()?).into()),
            Self::S => Arc::new(Self::S1(value.as_map()?).into()),
            Self::Index(index) => value
                .items()
                .get(index.as_usize()?)
                .ok_or_else(|| object_rainbow::error_operation!("index out of bounds"))?
                .clone(),
            Self::Unpack => {
                let InlineValue::Concat(a, b) = &*value else {
                    return Err(object_rainbow::error_operation!("not a tuple"));
                };
                let a = Self::K1(InlineDynamic::new(a.clone()));
                let b = Self::K1(InlineDynamic::new(b.clone()));
                let a = Self::S2(Arc::new(Self::I), Arc::new(a));
                Arc::new(Self::S2(Arc::new(a), Arc::new(b)).into())
            }
            Self::Pack1(a) => Arc::new(InlineValue::Concat(a.value(), value)),
            Self::Pack => Arc::new(Self::Pack1(InlineDynamic::new(value)).into()),
        })
    }
}

impl InlineMap {
    async fn apply_then_map(&self, value: Arc<InlineValue>) -> object_rainbow::Result<Arc<Self>> {
        self.apply(value).await?.as_map()
    }

    pub async fn apply(&self, value: Arc<InlineValue>) -> object_rainbow::Result<Arc<InlineValue>> {
        match self {
            #[cfg(feature = "point")]
            Self::Point(point) => Box::pin(point.fetch().await?.apply(value)).await,
            #[cfg(not(feature = "point"))]
            Self::Point(i) => match *i {},
            Self::S2(a, b) => {
                let (a, b) = try_join(
                    Box::pin(a.apply_then_map(value.clone())),
                    Box::pin(b.apply(value)),
                )
                .await?;
                Box::pin(a.apply(b)).await
            }
            _ => self.map(value),
        }
    }
}

#[cfg(feature = "apply")]
impl Apply<Arc<InlineValue>> for InlineMap {
    type Output = Arc<InlineValue>;

    async fn apply(&mut self, value: Arc<InlineValue>) -> object_rainbow::Result<Self::Output> {
        (*self).apply(value).await
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

impl From<InlineMap> for InlineValue {
    fn from(map: InlineMap) -> Self {
        Self::Map(Arc::new(map))
    }
}

impl InlineMap {
    pub fn s2(self: Arc<Self>, other: Arc<Self>) -> Arc<Self> {
        Arc::new(Self::S2(self, other))
    }

    pub fn s1(self: Arc<Self>) -> Arc<Self> {
        Arc::new(Self::S1(self))
    }

    pub fn k1(self: Arc<Self>) -> Arc<Self> {
        Arc::new(Self::K1(InlineDynamic::new(Arc::new(InlineValue::Map(
            self,
        )))))
    }

    fn swap_receiver() -> Arc<Self> {
        Arc::new(Self::Pack).s1().k1().s2(Arc::new(Self::K))
    }

    pub fn swap() -> Arc<Self> {
        Arc::new(Self::Unpack).s2(Self::swap_receiver().k1())
    }
}

#[test]
fn swap() -> object_rainbow::Result<()> {
    use object_rainbow::{ParseAs, ParseAsExtra};
    let schema = [5, 7, 0, 7, 0].as_slice().parse_as()?;
    let value: Arc<InlineValue> = [1, 2].as_slice().parse_as_extra(&schema)?;
    assert_eq!(value.vec(), [1, 2]);
    let value = InlineMap::swap().map(value)?;
    assert_eq!(value.vec(), [2, 1]);
    Ok(())
}

#[derive(Debug)]
pub enum MaybeFree {
    Apply(Arc<Self>, Arc<Self>),
    Refer(Arc<str>),
    Primitive(Arc<InlineMap>),
}

#[derive(Debug)]
pub enum MaybeLambda {
    Apply(Arc<Self>, Arc<Self>),
    Refer(Arc<str>),
    Define(Arc<str>, Arc<Self>),
    Primitive(Arc<InlineMap>),
}

impl MaybeFree {
    pub fn maybe_lambda(&self) -> Arc<MaybeLambda> {
        Arc::new(match self {
            Self::Apply(a, b) => MaybeLambda::Apply(a.maybe_lambda(), b.maybe_lambda()),
            Self::Refer(var) => MaybeLambda::Refer(var.clone()),
            Self::Primitive(primitive) => MaybeLambda::Primitive(primitive.clone()),
        })
    }
}

impl MaybeLambda {
    pub fn free(&self) -> BTreeSet<Arc<str>> {
        match self {
            Self::Apply(a, b) => a.free().union(&b.free()).cloned().collect(),
            Self::Refer(var) => [var.clone()].into(),
            Self::Define(var, definition) => {
                let mut free = definition.free();
                free.remove(var);
                free
            }
            Self::Primitive(_) => Default::default(),
        }
    }

    pub fn primitive(&self) -> Result<Arc<InlineMap>, Arc<MaybeFree>> {
        match self {
            Self::Define(var, definition)
                if let Self::Refer(refer) = &**definition
                    && var == refer =>
            {
                Ok(Arc::new(InlineMap::I))
            }
            Self::Define(var, definition) if !definition.free().contains(var) => Self::Apply(
                Arc::new(Self::Primitive(Arc::new(InlineMap::K))),
                definition.clone(),
            )
            .primitive(),
            Self::Define(var, definition)
                if let Self::Apply(a, b) = &**definition
                    && let Self::Refer(refer) = &**b
                    && var == refer
                    && !a.free().contains(var) =>
            {
                a.primitive()
            }
            Self::Define(var, definition) if let Self::Apply(a, b) = &**definition => Self::Apply(
                Arc::new(Self::Apply(
                    Arc::new(Self::Primitive(Arc::new(InlineMap::S))),
                    Arc::new(Self::Define(var.clone(), a.clone())),
                )),
                Arc::new(Self::Define(var.clone(), b.clone())),
            )
            .primitive(),
            Self::Define(var, definition) => match definition.primitive() {
                Ok(_) => unreachable!(),
                Err(definition) => Self::Define(var.clone(), definition.maybe_lambda()).primitive(),
            },
            Self::Apply(a, b) => match (a.primitive(), b.primitive()) {
                (Ok(a), Ok(b)) => Ok(a
                    .map(Arc::new(InlineValue::Map(b)))
                    .unwrap()
                    .as_map()
                    .unwrap()),
                (Ok(a), Err(b)) => Err(Arc::new(MaybeFree::Apply(
                    Arc::new(MaybeFree::Primitive(a)),
                    b,
                ))),
                (Err(a), Ok(b)) => Err(Arc::new(MaybeFree::Apply(
                    a,
                    Arc::new(MaybeFree::Primitive(b)),
                ))),
                (Err(a), Err(b)) => Err(Arc::new(MaybeFree::Apply(a, b))),
            },
            Self::Refer(var) => Err(Arc::new(MaybeFree::Refer(var.clone()))),
            Self::Primitive(primitive) => Ok(primitive.clone()),
        }
    }
}

#[test]
fn primitive() {
    let map = MaybeLambda::Define(
        "t".into(),
        Arc::new(MaybeLambda::Apply(
            Arc::new(MaybeLambda::Apply(
                Arc::new(MaybeLambda::Primitive(Arc::new(InlineMap::Unpack))),
                Arc::new(MaybeLambda::Refer("t".into())),
            )),
            Arc::new(MaybeLambda::Define(
                "a".into(),
                Arc::new(MaybeLambda::Define(
                    "b".into(),
                    Arc::new(MaybeLambda::Apply(
                        Arc::new(MaybeLambda::Apply(
                            Arc::new(MaybeLambda::Primitive(Arc::new(InlineMap::Pack))),
                            Arc::new(MaybeLambda::Refer("b".into())),
                        )),
                        Arc::new(MaybeLambda::Refer("a".into())),
                    )),
                )),
            )),
        )),
    )
    .primitive()
    .unwrap();
    assert_eq!(map, InlineMap::swap());
}
