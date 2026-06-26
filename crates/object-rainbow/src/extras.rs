use crate::*;

#[derive(Debug, Clone, ParseAsInline, PartialEq)]
pub struct Extras<Extra>(pub Extra);

impl<Extra> Deref for Extras<Extra> {
    type Target = Extra;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Extra> ToOutput for Extras<Extra> {
    fn to_output(&self, _: &mut impl Output) {}
}

impl<Extra> InlineOutput for Extras<Extra> {}

impl<I: PointInput> ParseInline<I> for Extras<I::Extra> {
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        Ok(Self(input.extra().clone()))
    }
}

impl<Extra> Tagged for Extras<Extra> {}
impl<Extra> ListHashes for Extras<Extra> {}
impl<Extra> Topological for Extras<Extra> {}
