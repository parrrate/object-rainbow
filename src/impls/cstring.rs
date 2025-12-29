use std::ffi::CString;

use crate::*;

impl ToOutput for CString {
    fn to_output(&self, output: &mut dyn Output) {
        self.as_c_str().to_output(output);
    }
}

impl Topological for CString {}
impl Tagged for CString {}
