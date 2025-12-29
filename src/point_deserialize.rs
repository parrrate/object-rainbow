use serde::{Deserialize, de::DeserializeOwned};

use super::*;

impl<'de, T: DeserializeOwned + Traversible + Clone> Deserialize<'de> for Point<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(T::deserialize(deserializer)?.point())
    }
}
