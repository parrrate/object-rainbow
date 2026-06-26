use std::iter::Zip;

use crate::{sequence::PlainCollection, *};

#[derive(ListHashes, Topological, Tagged, Size)]
pub struct TupleOfArrays<A, B>(A, B);

impl<A: ToOutput, B: ToOutput> ToOutput for TupleOfArrays<A, B> {
    fn to_output(&self, output: &mut impl crate::Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl<
    A: Parse<I> + PlainCollection<Item = Ae>,
    B: Parse<I> + PlainCollection<Item = Be>,
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

impl<A: IntoIterator, B: IntoIterator> IntoIterator for TupleOfArrays<A, B> {
    type Item = (A::Item, B::Item);

    type IntoIter = Zip<A::IntoIter, B::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().zip(self.1)
    }
}

impl<A: PlainCollection, B: PlainCollection> PlainCollection for TupleOfArrays<A, B> {}

impl<A: Extend<L>, B: Extend<R>, L, R> Extend<(L, R)> for TupleOfArrays<A, B> {
    fn extend<T: IntoIterator<Item = (L, R)>>(&mut self, iter: T) {
        let (a, b): (Vec<_>, Vec<_>) = iter.into_iter().collect();
        self.0.extend(a);
        self.1.extend(b);
    }
}
