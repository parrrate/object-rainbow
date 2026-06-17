use std::sync::Arc;

use crate::{with_repr::WithRepr, *};

pub struct Dt<T> {
    inner: Arc<WithRepr<T>>,
}

impl<T> Deref for Dt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.object()
    }
}

impl<T, A: Default + InlineOutput> ToOutput for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn to_output(&self, output: &mut impl Output) {
        self.iter_to_output(output);
        A::default().to_output(output);
    }
}

impl<T, A: Default + InlineOutput> InlineOutput for Dt<T> where
    for<'a> &'a T: IntoIterator<Item = &'a A>
{
}

impl<T, A: ListHashes> ListHashes for Dt<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a A>,
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}
