use object_rainbow::{length_prefixed::LpBytes, pod};
use object_rainbow_amt::AmtMap;

#[pod(no_copy, no_ord)]
pub struct RangeIndex<K> {
    prefixes: AmtMap<LpBytes, AmtMap<K, ()>>,
    filters: AmtMap<LpBytes, AmtMap<K, ()>>,
}
