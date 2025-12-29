use std::ffi::CString;

use crate::*;

impl ToOutput for CString {
    fn to_output(&self, output: &mut dyn Output) {
        self.as_c_str().to_output(output);
    }
}

impl Topological for CString {}
impl Tagged for CString {}

impl<I: ParseInput> Parse<I> for CString {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for CString {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let mut vec = Vec::new();
        while let x = input.parse_inline()?
            && x != 0
        {
            vec.push(x);
        }
        Ok(Self::new(vec).expect("should stop on zero"))
    }
}
