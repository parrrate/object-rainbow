use std::cmp::Ordering;

use crate::*;

/// Traits for which total order matches that of [`ToOutput::vec`].
pub trait ByteOrd: ToOutput + PartialOrd {
    fn bytes_cmp(&self, other: &Self) -> Ordering {
        self.vec().cmp(&other.vec())
    }
}

pub struct OrderedByBytes<T>(pub T);

impl<T: ByteOrd> PartialEq for OrderedByBytes<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.bytes_cmp(&other.0).is_eq()
    }
}

impl<T: ByteOrd> Eq for OrderedByBytes<T> {}

impl<T: ByteOrd> PartialOrd for OrderedByBytes<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ByteOrd> Ord for OrderedByBytes<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.bytes_cmp(&other.0)
    }
}

pub trait SignificantLength: ByteOrd {}
