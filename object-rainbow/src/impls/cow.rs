use std::borrow::Cow;

use crate::*;

impl<'a, B: 'a + ToOwned + ToOutput + ?Sized> ToOutput for Cow<'a, B> {
    fn to_output(&self, output: &mut dyn crate::Output) {
        (**self).to_output(output);
    }
}

impl<'a, B: 'a + ToOwned + InlineOutput + ?Sized> InlineOutput for Cow<'a, B> {}

impl<'a, B: 'a + ToOwned<Owned: Parse<I>> + ?Sized, I: ParseInput> Parse<I> for Cow<'a, B> {
    fn parse(input: I) -> crate::Result<Self> {
        input.parse().map(Self::Owned)
    }
}

impl<'a, B: 'a + ToOwned<Owned: ParseInline<I>> + ?Sized, I: ParseInput> ParseInline<I>
    for Cow<'a, B>
{
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        input.parse_inline().map(Self::Owned)
    }
}

impl<'a, B: 'a + ToOwned + ListHashes + ?Sized> ListHashes for Cow<'a, B> {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        (**self).list_hashes(f);
    }

    fn topology_hash(&self) -> Hash {
        (**self).topology_hash()
    }

    fn point_count(&self) -> usize {
        (**self).point_count()
    }
}

impl<'a, B: 'a + ToOwned + Topological + ?Sized> Topological for Cow<'a, B> {
    fn traverse(&self, visitor: &mut impl PointVisitor) {
        (**self).traverse(visitor);
    }

    fn topology(&self) -> TopoVec {
        (**self).topology()
    }
}

impl<'a, B: 'a + ToOwned + Tagged + ?Sized> Tagged for Cow<'a, B> {
    const TAGS: crate::Tags = B::TAGS;
    const HASH: crate::Hash = B::HASH;
}

impl<'a, B: 'a + ToOwned + Size + ?Sized> Size for Cow<'a, B> {
    const SIZE: usize = B::SIZE;
    type Size = B::Size;
}

impl<'a, B: 'a + ToOwned + MaybeHasNiche + ?Sized> MaybeHasNiche for Cow<'a, B> {
    type MnArray = B::MnArray;
}

impl<'a, B: 'a + ToOwned + ?Sized> Equivalent<B::Owned> for Cow<'a, B> {
    fn into_equivalent(self) -> B::Owned {
        self.into_owned()
    }

    fn from_equivalent(object: B::Owned) -> Self {
        Cow::Owned(object)
    }
}
