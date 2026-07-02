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
    Primitive(Arc<InlineValue>),
}

#[derive(Debug)]
pub enum MaybeLambda {
    Apply(Arc<Self>, Arc<Self>),
    Refer(Arc<str>),
    Define(Arc<str>, Arc<Self>),
    Primitive(Arc<InlineValue>),
}

#[macro_export]
macro_rules! inline_map {
    (I) => {
        InlineMap::I
    };
    (K) => {
        InlineMap::K
    };
    (S) => {
        InlineMap::S
    };
    (unpack) => {
        InlineMap::Unpack
    };
    (pack) => {
        InlineMap::Pack
    };
    (index($index:literal)) => {
        InlineMap::Index(U63::from_u64($index).unwrap())
    };
}

#[macro_export]
macro_rules! lambda {
    (($($a:tt)*) ($($b:tt)*)) => {
        Arc::new(MaybeLambda::Apply($crate::lambda!($($a)*), $crate::lambda!($($b)*)))
    };
    (($($a:tt)*) ($($b:tt)*) $($c:tt)*) => {
        lambda!((($($a)*)($($b)*))$($c)*)
    };
    ($var:literal) => {
        Arc::new(MaybeLambda::Refer($var.into()))
    };
    (|$var:literal| $($definition:tt)*) => {
        Arc::new(MaybeLambda::Define($var.into(), $crate::lambda!($($definition)*)))
    };
    (!$($primitive:tt)*) => {
        Arc::new(MaybeLambda::Primitive(Arc::new($crate::inline_map!($($primitive)*).into())))
    };
}
pub use lambda;

macro_rules! static_lambda {
    ($($l:tt)*) => {{
        static LAMBDA: std::sync::LazyLock<Arc<InlineMap>> = std::sync::LazyLock::new(
            || lambda!($($l)*).primitive().unwrap().as_map().unwrap()
        );
        LAMBDA.clone()
    }};
}

impl InlineMap {
    pub fn swap() -> Arc<Self> {
        static_lambda!(|"t"| (!unpack)("t")(|"a"| |"b"| (!pack)("b")("a")))
    }

    pub fn rotate_l() -> Arc<Self> {
        static_lambda!(
            |"abc"| (!unpack)("abc")(|"a"| |"bc"| (!unpack)("bc")(|"b"| |"c"| (!pack)((!pack)(
                "a"
            )(
                "b"
            ))("c")))
        )
    }

    pub fn rotate_r() -> Arc<Self> {
        static_lambda!(
            |"abc"| (!unpack)("abc")(|"ab"| |"c"| (!unpack)("ab")(|"a"| |"b"| (!pack)("a")(
                (!pack)("b")("c")
            )))
        )
    }

    pub fn map_l() -> Arc<Self> {
        static_lambda!(|"f"| |"ab"| (!unpack)("ab")(|"a"| |"b"| (!pack)(("f")("a"))("b")))
    }
}

#[test]
fn rotate_l() -> object_rainbow::Result<()> {
    use crate::AbstractValue;
    use object_rainbow::{ParseAs, ParseAsExtra};
    let schema = [5, 7, 0, 5, 7, 0, 7, 0].as_slice().parse_as()?;
    let value: Arc<InlineValue> = [1, 2, 3].as_slice().parse_as_extra(&schema)?;
    assert_eq!(value.vec(), [1, 2, 3]);
    let value = InlineMap::rotate_l().map(value)?;
    assert_eq!(value.vec(), [1, 2, 3]);
    assert_eq!(value.schema().vec(), [5, 5, 7, 0, 7, 0, 7, 0]);
    Ok(())
}

#[test]
fn rotate_r() -> object_rainbow::Result<()> {
    use crate::AbstractValue;
    use object_rainbow::{ParseAs, ParseAsExtra};
    let schema = [5, 5, 7, 0, 7, 0, 7, 0].as_slice().parse_as()?;
    let value: Arc<InlineValue> = [1, 2, 3].as_slice().parse_as_extra(&schema)?;
    assert_eq!(value.vec(), [1, 2, 3]);
    let value = InlineMap::rotate_r().map(value)?;
    assert_eq!(value.vec(), [1, 2, 3]);
    assert_eq!(value.schema().vec(), [5, 7, 0, 5, 7, 0, 7, 0]);
    Ok(())
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

    pub fn free_var(&self, var: &Arc<str>) -> bool {
        match self {
            Self::Apply(a, b) => a.free_var(var) || b.free_var(var),
            Self::Refer(refer) => var == refer,
            Self::Define(refer, definition) => var != refer && definition.free_var(var),
            Self::Primitive(_) => false,
        }
    }

    pub fn primitive(&self) -> Result<Arc<InlineValue>, Arc<MaybeFree>> {
        match self {
            Self::Define(var, definition)
                if let Self::Refer(refer) = &**definition
                    && var == refer =>
            {
                Ok(Arc::new(InlineMap::I.into()))
            }
            Self::Define(var, definition) if !definition.free_var(var) => Self::Apply(
                Arc::new(Self::Primitive(Arc::new(InlineMap::K.into()))),
                definition.clone(),
            )
            .primitive(),
            Self::Define(var, definition)
                if let Self::Apply(a, b) = &**definition
                    && let Self::Refer(refer) = &**b
                    && var == refer
                    && !a.free_var(var) =>
            {
                a.primitive()
            }
            Self::Define(var, definition) if let Self::Apply(a, b) = &**definition => Self::Apply(
                Arc::new(Self::Apply(
                    Arc::new(Self::Primitive(Arc::new(InlineMap::S.into()))),
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
                (Ok(a), Ok(b)) => a.as_map().and_then(|a| a.map(b.clone())).map_err(|_| {
                    Arc::new(MaybeFree::Apply(
                        Arc::new(MaybeFree::Primitive(a)),
                        Arc::new(MaybeFree::Primitive(b)),
                    ))
                }),
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
