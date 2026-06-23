#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

use std::{ops::Deref, sync::Arc};

use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Output, Parse, ParseInput, Size, SomeNiche, Tagged,
    ToOutput, Topological, TryDefault, ZeroNiche,
};
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "distributed")]
pub use self::distributed::{Distributed, DistributedParseError};

#[cfg(feature = "distributed")]
mod distributed;

#[derive(Debug)]
struct JsonInner<T> {
    value: T,
    data: Vec<u8>,
}

impl<T> PartialEq for JsonInner<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for JsonInner<T> {}

impl<T> std::hash::Hash for JsonInner<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Json<T> {
    inner: Arc<JsonInner<T>>,
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.value
    }
}

impl<T> Clone for Json<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Serialize> Json<T> {
    pub fn new(value: T) -> object_rainbow::Result<Self> {
        let data = serde_json::to_vec(&value).map_err(object_rainbow::Error::parse)?;
        Ok(Self {
            inner: Arc::new(JsonInner { value, data }),
        })
    }
}

impl<T: Serialize + Default> TryDefault for Json<T> {
    fn try_default() -> object_rainbow::Result<Self> {
        Self::new(Default::default())
    }
}

impl<T: Serialize> ToOutput for Json<T> {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_real() {
            serde_json::to_writer(&mut output.as_write(), &self.inner.value)
                .expect("json write errors are considered bugs");
        }
    }
}

impl<T: DeserializeOwned + Serialize, I: ParseInput> Parse<I> for Json<T> {
    fn parse(input: I) -> object_rainbow::Result<Self> {
        let data = input.parse_all()?;
        let json = serde_json::from_slice(&data)
            .map_err(object_rainbow::Error::parse)
            .and_then(Self::new)?;
        if *data == json.vec() {
            Ok(json)
        } else {
            Err(object_rainbow::error_parse!("inconsistent serialization"))
        }
    }
}

impl<T> ListHashes for Json<T> {}
impl<T> Topological for Json<T> {}
impl<T> Tagged for Json<T> {}

impl InlineOutput for Json<()> {}

impl Size for Json<()> {
    type Size = object_rainbow::typenum::consts::U4;
}

impl MaybeHasNiche for Json<()> {
    type MnArray = SomeNiche<ZeroNiche<<Self as Size>::Size>>;
}
