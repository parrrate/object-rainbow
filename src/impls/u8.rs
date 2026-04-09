use crate::ToOutput;

impl ToOutput for u8 {
    fn to_output(&self, output: &mut dyn crate::Output) {
        output.write(&[*self]);
    }

    fn slice_to_output(slice: &[Self], output: &mut dyn crate::Output)
    where
        Self: Sized,
    {
        output.write(slice);
    }
}
