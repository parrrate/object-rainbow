use serde::Serialize;

use crate::Point;

impl<T: Serialize + Clone> Serialize for Point<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(object) = self.get() {
            return object.serialize(serializer);
        }
        if let Some((object, _)) = self
            .try_fetch_local()
            .map_err(<S::Error as serde::ser::Error>::custom)?
        {
            return object.serialize(serializer);
        }
        Err(<S::Error as serde::ser::Error>::custom(
            "cannot serialize remote Point",
        ))
    }
}
