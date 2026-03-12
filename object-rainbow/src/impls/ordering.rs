use std::cmp::Ordering;

use crate::{InlineOutput, ToOutput};

impl ToOutput for Ordering {
    fn to_output(&self, output: &mut dyn crate::Output) {
        (*self as i8).to_output(output);
    }
}

impl InlineOutput for Ordering {}
