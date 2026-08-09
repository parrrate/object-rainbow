use crate::{refless::assert_refless::AssertRefless, *};

pub trait ParseFetch<E: 'static + Clone>: Sized {
    fn parse_fetch<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self>;
}

pub trait ParseFetchInline<E: 'static + Clone>: ParseFetch<E> {
    fn parse_fetch_inline<I: PointInput<Extra: Fetch<T = E>>>(input: &mut I) -> Result<Self>;

    fn parse_fetch_as_inline<I: PointInput<Extra: Fetch<T = E>>>(input: I) -> Result<Self> {
        input.parse_as_inline(Self::parse_fetch_inline)
    }
}

pub struct FetchExtra<F>(pub F);

impl<
    E: 'static + Clone,
    F: ParseFetch<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> Parse<I> for FetchExtra<F>
{
    fn parse(input: I) -> crate::Result<Self> {
        F::parse_fetch(input).map(Self)
    }
}

impl<
    E: 'static + Clone,
    F: ParseFetchInline<E>,
    X: 'static + Clone + Fetch<T = E>,
    I: PointInput<Extra = X>,
> ParseInline<I> for FetchExtra<F>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        F::parse_fetch_inline(input).map(Self)
    }
}

pub struct DelayedRefless<T> {
    data: Vec<u8>,
    fetch: Arc<dyn Fetch<T = AssertRefless<T>>>,
}

impl<T> ToOutput for DelayedRefless<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.data.to_output(output);
    }
}

impl<T> ListHashes for DelayedRefless<T> {}
impl<T> Topological for DelayedRefless<T> {}

impl<T: ReflessObject + Clone> DelayedRefless<T> {
    pub async fn fetch(&self) -> object_rainbow::Result<T> {
        Ok(self.fetch.fetch().await?.0)
    }

    pub fn new(refless: T) -> Self {
        Self {
            data: refless.vec(),
            fetch: AssertRefless(refless).local_fetch(),
        }
    }
}
