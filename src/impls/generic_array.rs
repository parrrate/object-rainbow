use crate::*;

impl<T: InlineOutput, N: ArrayLength> ToOutput for GenericArray<T, N> {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}

impl<T: InlineOutput, N: ArrayLength> InlineOutput for GenericArray<T, N> {}

impl<T: ListHashes, N: ArrayLength> ListHashes for GenericArray<T, N> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.iter_list_hashes(f);
    }
}
