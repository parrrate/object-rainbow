use object_rainbow_derive::{ParseAsInline, Tagged, Topological};

use crate::*;

#[derive(Tagged, Topological)]
struct ZtInner<T> {
    object: T,
    data: Vec<u8>,
}

#[derive(Tagged, Topological, ParseAsInline)]
pub struct Zt<T> {
    inner: Arc<ZtInner<T>>,
}

impl<T> Clone for Zt<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for Zt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.object
    }
}

impl<T: ToOutput> ToOutput for Zt<T> {
    fn to_output(&self, output: &mut dyn Output) {
        self.inner.data.to_output(output);
        output.write(&[0]);
    }
}

impl<T: Parse<I>, I: ParseInput> ParseInline<I> for Zt<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let data = input.parse_until_zero()?;
        let object = input.reparse(data)?;
        let data = data.into();
        let inner = Arc::new(ZtInner { object, data });
        Ok(Self { inner })
    }
}

impl<T: Object<Extra>, Extra: 'static> Object<Extra> for Zt<T> {}
impl<T: Object<Extra>, Extra: 'static> Inline<Extra> for Zt<T> {}
impl<T: ReflessObject> ReflessObject for Zt<T> {}
impl<T: ReflessObject> ReflessInline for Zt<T> {}
