use std::result::Result;

use object_rainbow::Traversible;
use serde::{Deserialize, de::DeserializeOwned};

use crate::{IntoPoint, Point};

impl<'de, T: DeserializeOwned + Traversible + Clone> Deserialize<'de> for Point<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(T::deserialize(deserializer)?.point())
    }
}
