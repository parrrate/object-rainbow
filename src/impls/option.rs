use crate::*;

impl<T: ToOutput + MaybeHasNiche<MnArray: MnArray<MaybeNiche = N>>, N: Niche<NeedsTag = B>, B: Bit>
    ToOutput for Option<T>
{
    fn to_output(&self, output: &mut dyn Output) {
        match self {
            Some(value) => {
                if B::BOOL {
                    output.write(&[0]);
                }
                value.to_output(output);
            }
            None => {
                if B::BOOL {
                    output.write(&[1]);
                }
                output.write(N::niche().as_slice());
            }
        }
    }
}

impl<T: Topological> Topological for Option<T> {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.iter_accept_points(visitor);
    }
}

impl<T: Tagged> Tagged for Option<T> {
    const TAGS: Tags = T::TAGS;
}

impl<
    T: MaybeHasNiche<MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B>>, Size: Add<B, Output = N>>,
    B: Bit,
    N: Unsigned,
> Size for Option<T>
{
    type Size = N;
}

pub struct OptionNiche<N, I>(N, I);

impl<N: ArrayLength, I: Bit> Niche for OptionNiche<N, I> {
    type NeedsTag = I;
    type N = N;
    fn niche() -> GenericArray<u8, Self::N> {
        let mut niche = GenericArray::default();
        if !I::BOOL {
            niche[0] = 2;
        }
        niche
    }
}

impl<
    T: MaybeHasNiche<
            MnArray: MnArray<MaybeNiche: Niche<NeedsTag = B>>,
            Size: Add<B, Output: ArrayLength>,
        >,
    B: Bit + Not<Output = I>,
    I: Bit,
> MaybeHasNiche for Option<T>
{
    type MnArray = SomeNiche<OptionNiche<Self::Size, I>>;
}
