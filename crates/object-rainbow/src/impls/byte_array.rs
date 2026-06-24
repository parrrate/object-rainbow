use typenum::ToUInt;

use crate::*;

impl<const N: usize> MaybeHasNiche for [u8; N]
where
    typenum::generic_const_mappings::Const<N>: ToUInt<Output: ArrayLength>,
{
    type MnArray = NoNiche<ZeroNoNiche<typenum::generic_const_mappings::U<N>>>;
}

#[test]
fn byte_array_niche() {
    assert_eq!(None::<([u8; 2], bool)>.vec(), [0, 0, 2]);
}
