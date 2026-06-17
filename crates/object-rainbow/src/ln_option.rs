use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LnOption<T>(pub Option<T>);

impl<T: ByteOrd> ByteOrd for LnOption<T> {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.bytes_cmp(b),
        }
    }
}

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
