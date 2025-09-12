use crate::*;

impl ToOutput for () {
    fn to_output(&self, _: &mut dyn Output) {}
}

impl Topological for () {
    fn accept_points(&self, _: &mut impl PointVisitor) {}
}

impl<I: ParseInput> Parse<I> for () {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for () {
    fn parse_inline(_: &mut I) -> crate::Result<Self> {
        Ok(())
    }
}

impl Tagged for () {}

impl Object for () {}

impl Inline for () {}

impl ReflessObject for () {}

impl ReflessInline for () {}

impl Size for () {
    const SIZE: usize = 0;
    type Size = typenum::generic_const_mappings::U<{ Self::SIZE }>;
}

impl MaybeHasNiche for () {
    type MnArray = NoNiche<Self::Size>;
}
