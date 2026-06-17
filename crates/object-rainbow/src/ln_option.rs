use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LnOption<T>(pub Option<T>);

impl<T: ToOutput> ToOutput for LnOption<T> {
    fn to_output(&self, output: &mut impl Output) {
        if let Some(object) = &self.0 {
            if output.is_real() {
                output.write(&[0xff]);
            }
            object.to_output(output);
        } else if output.is_real() {
            output.write(&[0xfe]);
        }
    }
}

impl<T: InlineOutput> InlineOutput for LnOption<T> {}
