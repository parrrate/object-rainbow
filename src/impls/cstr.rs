use std::ffi::CStr;

use crate::*;

impl ToOutput for CStr {
    fn to_output(&self, output: &mut dyn Output) {
        self.to_bytes_with_nul().to_output(output);
    }
}
