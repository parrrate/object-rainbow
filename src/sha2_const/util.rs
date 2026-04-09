#[inline(always)]
pub(in crate::sha2_const) const fn memcpy(
    dest: &mut [u8],
    dest_offset: usize,
    src: &[u8],
    src_offset: usize,
    n: usize,
) {
    let mut i = 0;
    while i < n {
        dest[dest_offset + i] = src[src_offset + i];
        i += 1;
    }
}

#[inline(always)]
pub(in crate::sha2_const) const fn memset(dest: &mut [u8], offset: usize, val: u8, n: usize) {
    let mut i = 0;
    while i < n {
        dest[offset + i] = val;
        i += 1;
    }
}

#[inline(always)]
pub(in crate::sha2_const) const fn load_u32_be(src: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        src[offset],
        src[offset + 1],
        src[offset + 2],
        src[offset + 3],
    ])
}

#[inline(always)]
pub(in crate::sha2_const) const fn store_u32_be(dest: &mut [u8], offset: usize, n: u32) {
    let bytes = u32::to_be_bytes(n);
    memcpy(dest, offset, &bytes, 0, bytes.len());
}

#[inline(always)]
pub(in crate::sha2_const) const fn store_u64_be(dest: &mut [u8], offset: usize, n: u64) {
    let bytes = u64::to_be_bytes(n);
    memcpy(dest, offset, &bytes, 0, bytes.len());
}
