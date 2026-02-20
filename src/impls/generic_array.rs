use crate::*;

impl<T: InlineOutput, N: ArrayLength> ToOutput for GenericArray<T, N> {
    fn to_output(&self, output: &mut dyn Output) {
        T::slice_to_output(self, output);
    }
}
