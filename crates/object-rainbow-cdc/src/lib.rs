use object_rainbow::{DiffHashes, Hash, SizeExt, ToOutput};
use sha2::{Digest, Sha256};
use static_assertions::const_assert_eq;

pub fn generate_tail(data: &[u8]) -> u128 {
    let diff = DiffHashes::default().data_hash();
    let mut hasher = Sha256::new();
    hasher.update(diff);
    hasher.update(data);
    let hasher = hasher;
    let len: u32 = data.len().try_into().unwrap();
    let target = len >> 16;
    for tail in 0..u128::MAX {
        let mut hasher = hasher.clone();
        hasher.update(tail.to_be_bytes());
        if derive_length_from_hash(Hash::from_hasher(hasher)) == target {
            return tail;
        }
    }
    panic!("took too long")
}

pub fn derive_length_from_hash(hash: Hash) -> u32 {
    derive_length(hash.reinterpret::<(u64, u64, u64, u64)>().0)
}

pub fn derive_length(source: u64) -> u32 {
    const SIZE: u32 = u64::BITS;
    const HEAD_SIZE: u32 = 4;
    const TAIL_SIZE: u32 = SIZE - HEAD_SIZE;
    const TAIL_MASK: u64 = (1 << TAIL_SIZE) - 1;
    const BASE_BITS: u32 = 1;
    const MAX_BITS: u32 = BASE_BITS + ((1 << HEAD_SIZE) - 1) - 1;
    const MAX_GARBAGE: u32 = TAIL_SIZE - BASE_BITS;
    const_assert_eq!(MAX_BITS, 15);
    let head = (source >> TAIL_SIZE) as u32;
    let tail = source & TAIL_MASK;
    let garbage = MAX_GARBAGE - head.saturating_sub(1);
    let extra = (tail >> garbage) as u32;
    let main = if head == 0 {
        0
    } else {
        1 << (BASE_BITS - 1 + head)
    };
    main | extra
}

#[test]
#[expect(clippy::unusual_byte_groupings)]
fn cases() {
    let f = derive_length;
    assert_eq!(
        f(0b_0000_0_11111111111111111111111111111111111111111111111111111111111),
        0b_0,
    );
    assert_eq!(
        f(0b_0000_1_00000000000000000000000000000000000000000000000000000000000),
        0b_1,
    );
    assert_eq!(
        f(0b_0001_0_11111111111111111111111111111111111111111111111111111111111),
        0b_10,
    );
    assert_eq!(
        f(0b_0001_1_00000000000000000000000000000000000000000000000000000000000),
        0b_11,
    );
    assert_eq!(
        f(0b_0010_00_1111111111111111111111111111111111111111111111111111111111),
        0b_100,
    );
    assert_eq!(
        f(0b_0010_11_0000000000000000000000000000000000000000000000000000000000),
        0b_111,
    );
    assert_eq!(
        f(0b_0100_0000_11111111111111111111111111111111111111111111111111111111),
        0b_10000,
    );
    assert_eq!(
        f(0b_0100_1111_00000000000000000000000000000000000000000000000000000000),
        0b_11111,
    );
    assert_eq!(
        f(0b_1000_00000000_1111111111111111111111111111111111111111111111111111),
        0b_100000000,
    );
    assert_eq!(
        f(0b_1000_11111111_0000000000000000000000000000000000000000000000000000),
        0b_111111111,
    );
    assert_eq!(
        f(0b_1111_000000000000000_111111111111111111111111111111111111111111111),
        0b_1000000000000000,
    );
    assert_eq!(
        f(0b_1111_111111111111111_000000000000000000000000000000000000000000000),
        0b_1111111111111111,
    );
}
