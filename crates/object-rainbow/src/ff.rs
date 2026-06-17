use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ListHashes, Topological)]
pub struct Ff;

impl ToOutput for Ff {
    fn to_output(&self, output: &mut impl Output) {
        output.write(&[0xff]);
    }
}
