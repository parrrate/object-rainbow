use std::ffi::CStr;

use crate::*;

impl ToOutput for CStr {
    fn to_output(&self, output: &mut dyn Output) {
        self.to_bytes_with_nul().to_output(output);
    }
}

impl InlineOutput for CStr {}

impl ListHashes for CStr {}
impl Topological for CStr {}
impl Tagged for CStr {}
