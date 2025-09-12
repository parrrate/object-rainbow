use crate::*;

impl ToOutput for bool {
    fn to_output(&self, output: &mut dyn Output) {
        output.write(&[*self as _])
    }
}

impl Topological for bool {
    fn accept_points(&self, _: &mut impl PointVisitor) {}
}

impl Tagged for bool {}

impl Size for bool {
    type Size = U1;
}

pub struct BoolNiche;

impl Niche for BoolNiche {
    type NeedsTag = B0;
    type N = U1;
    fn niche() -> GenericArray<u8, Self::N> {
        GenericArray::from_array([2])
    }
}

impl MaybeHasNiche for bool {
    type MnArray = SomeNiche<BoolNiche>;
}
