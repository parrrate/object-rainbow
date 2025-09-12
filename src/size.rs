pub const fn equal_merge(a: usize, b: usize) -> usize {
    if a != b {
        panic!("inequal lengths, cannot decide `enum` size")
    }
    a
}
