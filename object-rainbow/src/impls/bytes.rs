use typenum::ToUInt;

use crate::*;

impl<const N: usize> MaybeHasNiche for [u8; N]
where
    typenum::generic_const_mappings::Const<N>: ToUInt<Output: ArrayLength>,
{
    type MnArray = NoNiche<ZeroNoNiche<typenum::generic_const_mappings::U<N>>>;
}
