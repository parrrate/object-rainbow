use std::cmp::Ordering;

use crate::*;

impl ToOutput for Ordering {
    fn to_output(&self, output: &mut dyn crate::Output) {
        (*self as i8).to_output(output);
    }
}

impl InlineOutput for Ordering {}
impl Tagged for Ordering {}
impl ListHashes for Ordering {}
impl Topological for Ordering {}
