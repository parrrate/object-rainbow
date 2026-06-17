use std::sync::Arc;

use crate::{with_repr::WithRepr, *};

pub struct Dt<T> {
    inner: Arc<WithRepr<T>>,
}

impl<T, A: Default + InlineOutput> ToOutput for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.inner.object().iter_to_output(output);
        A::default().to_output(output);
    }
}
