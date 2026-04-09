use crate::*;

#[derive(Debug, Tagged, ListHashes, Topological, Clone, Copy, Default, ParseAsInline)]
pub struct ParseExtra<T>(pub T);

impl<
    T: ParseSliceExtra<B>,
    I: PointInput<Extra = (A, B)>,
    A: 'static + Clone + AsRef<[u8]>,
    B: 'static + Clone,
> ParseInline<I> for ParseExtra<T>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        T::parse_slice_extra(
            input.extra().0.as_ref(),
            &input.resolve(),
            &input.extra().1.clone(),
        )
        .map(Self)
    }
}
