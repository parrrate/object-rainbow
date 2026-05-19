use std::cmp::Ordering;

use crate::*;

/// Traits for which total order matches that of [`ToOutput::vec`].
pub trait ByteOrdered: ToOutput + PartialOrd {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.vec().cmp(&other.vec())
    }
}

#[derive(PartialEq)]
pub struct OrderedByBytes<T>(pub T);
