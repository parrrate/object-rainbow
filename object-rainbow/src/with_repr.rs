use crate::*;

#[derive(Debug, Tagged, ListHashes, Topological)]
pub struct WithRepr<T> {
    object: T,
    data: Vec<u8>,
}

impl<T: ToOutput> ToOutput for WithRepr<T> {
    fn to_output(&self, output: &mut impl Output) {
        if output.is_mangling() {
            self.object.to_output(output);
        }
        if output.is_real() {
            self.data.to_output(output);
        }
    }
}

impl<T> WithRepr<T> {
    pub fn new(object: T) -> Self
    where
        T: ToOutput,
    {
        let data = object.vec();
        Self { object, data }
    }

    pub fn parse_zero_terminated<I: ParseInput>(input: &mut I) -> crate::Result<Self>
    where
        T: Parse<I>,
    {
        let (data, object) = input.parse_zero_terminated()?;
        Ok(Self { object, data })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn object(&self) -> &T {
        &self.object
    }
}

impl<T> PartialEq for WithRepr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for WithRepr<T> {}

impl<T> PartialOrd for WithRepr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for WithRepr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T: Parse<I> + ToOutput, I: ParseInput> Parse<I> for WithRepr<T> {
    fn parse(input: I) -> crate::Result<Self> {
        Ok(Self::new(input.parse()?))
    }
}

impl<T: ParseInline<I> + ToOutput, I: ParseInput> ParseInline<I> for WithRepr<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        Ok(Self::new(input.parse_inline()?))
    }
}

impl<T: Size> Size for WithRepr<T> {
    type Size = T::Size;
    const SIZE: usize = T::SIZE;
}

impl<T: MaybeHasNiche> MaybeHasNiche for WithRepr<T> {
    type MnArray = WithRepr<T>;
}
