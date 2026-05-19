use std::cmp::Ordering;

use crate::*;

/// Traits for which total order matches that of [`ToOutput::vec`].
pub trait ByteOrdered: ToOutput {
    fn bytes_cmp(&self, right: &Self) -> Ordering {
        self.vec().cmp(&right.vec())
    }
}
