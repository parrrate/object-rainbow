use object_rainbow_derive::{ParseAsInline, Tagged, Topological};

use crate::*;

#[derive(Tagged, Topological, ParseAsInline)]
pub struct Zt<T> {
    object: Arc<T>,
    data: Arc<Vec<u8>>,
}

impl<T> Clone for Zt<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T: ToOutput> ToOutput for Zt<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.data.to_output(output);
        output.write(&[0]);
    }
}

impl<T: Parse<I>, I: ParseInput> ParseInline<I> for Zt<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let data = input.parse_until_zero()?;
        let object = input.reparse(data)?;
        let data = data.into().into();
        Ok(Self { object, data })
    }
}

impl<T: Object<Extra>, Extra: 'static> Object<Extra> for Zt<T> {}
impl<T: Object<Extra>, Extra: 'static> Inline<Extra> for Zt<T> {}
impl<T: ReflessObject> ReflessObject for Zt<T> {}
impl<T: ReflessObject> ReflessInline for Zt<T> {}
