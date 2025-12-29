use serde::Serialize;

use crate::Point;

impl<T: Serialize + Clone> Serialize for Point<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get()
            .ok_or_else(|| {
                <S::Error as serde::ser::Error>::custom("cannot serialize remote Point")
            })?
            .serialize(serializer)
    }
}
