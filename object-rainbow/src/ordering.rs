use std::cmp::Ordering;

use crate::*;

/// Traits for which total order matches that of [`ToOutput::vec`].
pub trait ByteOrdered: ToOutput + PartialOrd {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.vec().cmp(&other.vec())
    }
}

pub struct OrderedByBytes<T>(pub T);

impl<T: ByteOrdered> PartialEq for OrderedByBytes<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.bytes_cmp(&other.0).is_eq()
    }
}
