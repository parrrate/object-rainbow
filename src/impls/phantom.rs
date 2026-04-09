use crate::*;

impl<T: ?Sized> ToOutput for PhantomData<T> {
    fn to_output(&self, _: &mut dyn Output) {}
}

impl<T: ?Sized, I: ParseInput> Parse<I> for PhantomData<T> {
    fn parse(input: I) -> crate::Result<Self> {
        ParseInline::parse_as_inline(input)
    }
}

impl<T: ?Sized, I: ParseInput> ParseInline<I> for PhantomData<T> {
    fn parse_inline(_: &mut I) -> crate::Result<Self> {
        Ok(Self)
    }
}

impl<T: ?Sized> Topological for PhantomData<T> {}
impl<T: ?Sized> Tagged for PhantomData<T> {}
impl<T: ?Sized + 'static + Send + Sync, E> Object<E> for PhantomData<T> {}
impl<T: ?Sized + 'static + Send + Sync, E> Inline<E> for PhantomData<T> {}
impl<T: ?Sized + 'static + Send + Sync> ReflessObject for PhantomData<T> {}
impl<T: ?Sized + 'static + Send + Sync> ReflessInline for PhantomData<T> {}

impl<T: ?Sized> Size for PhantomData<T> {
    const SIZE: usize = 0;
    type Size = typenum::U0;
}

impl<T: ?Sized> MaybeHasNiche for PhantomData<T> {
    type MnArray = NoNiche<ZeroNoNiche<<Self as Size>::Size>>;
}
