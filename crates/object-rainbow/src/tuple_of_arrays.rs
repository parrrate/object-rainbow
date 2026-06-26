use crate::*;

#[derive(ListHashes, Topological, Tagged, Size)]
pub struct TupleOfArrays<A, B>(A, B);

impl<A: ToOutput, B: ToOutput> ToOutput for TupleOfArrays<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl<
    A: Parse<I> + IntoIterator<Item = Ae>,
    B: Parse<I> + IntoIterator<Item = Be>,
    I: ParseInput,
    Ae: Size,
    Be: Size,
> Parse<I> for TupleOfArrays<A, B>
{
    fn parse(input: I) -> crate::Result<Self> {
        let (mut input, n) = input.remaining()?;
        let k = Ae::SIZE + Be::SIZE;
        if !n.is_multiple_of(k) {
            return Err(crate::error_parse!("doesn't divide evenly"));
        }
        let n = n
            .checked_div(k)
            .ok_or_else(|| crate::error_parse!("division failed"))?;
        Ok(Self(input.split_parse(n * Ae::SIZE)?, input.parse()?))
    }
}

pub trait VecLike: IntoIterator {}
