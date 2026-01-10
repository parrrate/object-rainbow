use typenum::tarr;

use crate::*;

impl<A: InlineOutput, B: ToOutput> ToOutput for (A, B) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl<A: InlineOutput, B: InlineOutput> InlineOutput for (A, B) {}

impl<A: ListHashes, B: ListHashes> ListHashes for (A, B) {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
    }
}

impl<A: Topological, B: Topological> Topological for (A, B) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged> Tagged for (A, B) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS]);
}

impl<A: Size, B: Size> Size for (A, B)
where
    tarr![A::Size, B::Size,]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE;

    type Size = <tarr![A::Size, B::Size,] as typenum::FoldAdd>::Output;
}

impl<II: ParseInput, A: ParseInline<II>, B: Parse<II>> Parse<II> for (A, B) {
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse()?))
    }
}

impl<II: ParseInput, A: ParseInline<II>, B: ParseInline<II>> ParseInline<II> for (A, B) {
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?))
    }
}

impl<A: MaybeHasNiche, B: MaybeHasNiche> MaybeHasNiche for (A, B) {
    type MnArray = tarr![A::MnArray, B::MnArray,];
}

impl<A: InlineOutput, B: InlineOutput, C: ToOutput> ToOutput for (A, B, C) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
    }
}

impl<A: InlineOutput, B: InlineOutput, C: InlineOutput> InlineOutput for (A, B, C) {}

impl<A: ListHashes, B: ListHashes, C: ListHashes> ListHashes for (A, B, C) {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
    }
}

impl<A: Topological, B: Topological, C: Topological> Topological for (A, B, C) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged> Tagged for (A, B, C) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS]);
}

impl<A: Size, B: Size, C: Size> Size for (A, B, C)
where
    tarr![A::Size, B::Size, C::Size,]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE;

    type Size = <tarr![A::Size, B::Size, C::Size,] as typenum::FoldAdd>::Output;
}

impl<II: ParseInput, A: ParseInline<II>, B: ParseInline<II>, C: Parse<II>> Parse<II> for (A, B, C) {
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?, input.parse()?))
    }
}

impl<II: ParseInput, A: ParseInline<II>, B: ParseInline<II>, C: ParseInline<II>> ParseInline<II>
    for (A, B, C)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: MaybeHasNiche, B: MaybeHasNiche, C: MaybeHasNiche> MaybeHasNiche for (A, B, C) {
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray,];
}

impl<A: InlineOutput, B: InlineOutput, C: InlineOutput, D: ToOutput> ToOutput for (A, B, C, D) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
    }
}

impl<A: InlineOutput, B: InlineOutput, C: InlineOutput, D: InlineOutput> InlineOutput
    for (A, B, C, D)
{
}

impl<A: ListHashes, B: ListHashes, C: ListHashes, D: ListHashes> ListHashes for (A, B, C, D) {
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
    }
}

impl<A: Topological, B: Topological, C: Topological, D: Topological> Topological for (A, B, C, D) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged> Tagged for (A, B, C, D) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS]);
}

impl<A: Size, B: Size, C: Size, D: Size> Size for (A, B, C, D)
where
    tarr![A::Size, B::Size, C::Size, D::Size,]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE;

    type Size = <tarr![A::Size, B::Size, C::Size, D::Size,] as typenum::FoldAdd>::Output;
}

impl<II: ParseInput, A: ParseInline<II>, B: ParseInline<II>, C: ParseInline<II>, D: Parse<II>>
    Parse<II> for (A, B, C, D)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<II: ParseInput, A: ParseInline<II>, B: ParseInline<II>, C: ParseInline<II>, D: ParseInline<II>>
    ParseInline<II> for (A, B, C, D)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: MaybeHasNiche, B: MaybeHasNiche, C: MaybeHasNiche, D: MaybeHasNiche> MaybeHasNiche
    for (A, B, C, D)
{
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray,];
}

impl<A: InlineOutput, B: InlineOutput, C: InlineOutput, D: InlineOutput, E: ToOutput> ToOutput
    for (A, B, C, D, E)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
    }
}

impl<A: InlineOutput, B: InlineOutput, C: InlineOutput, D: InlineOutput, E: InlineOutput>
    InlineOutput for (A, B, C, D, E)
{
}

impl<A: ListHashes, B: ListHashes, C: ListHashes, D: ListHashes, E: ListHashes> ListHashes
    for (A, B, C, D, E)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
    }
}

impl<A: Topological, B: Topological, C: Topological, D: Topological, E: Topological> Topological
    for (A, B, C, D, E)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged, E: Tagged> Tagged for (A, B, C, D, E) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS, &E::TAGS]);
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size> Size for (A, B, C, D, E)
where
    tarr![A::Size, B::Size, C::Size, D::Size, E::Size,]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE;

    type Size = <tarr![A::Size, B::Size, C::Size, D::Size, E::Size,] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: Parse<II>,
> Parse<II> for (A, B, C, D, E)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: MaybeHasNiche, B: MaybeHasNiche, C: MaybeHasNiche, D: MaybeHasNiche, E: MaybeHasNiche>
    MaybeHasNiche for (A, B, C, D, E)
{
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray, E::MnArray,];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: ToOutput,
> ToOutput for (A, B, C, D, E, F)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
> InlineOutput for (A, B, C, D, E, F)
{
}

impl<A: ListHashes, B: ListHashes, C: ListHashes, D: ListHashes, E: ListHashes, F: ListHashes>
    ListHashes for (A, B, C, D, E, F)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
    }
}

impl<A: Topological, B: Topological, C: Topological, D: Topological, E: Topological, F: Topological>
    Topological for (A, B, C, D, E, F)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged, E: Tagged, F: Tagged> Tagged
    for (A, B, C, D, E, F)
{
    const TAGS: Tags = Tags(
        &[],
        &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS, &E::TAGS, &F::TAGS],
    );
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size> Size for (A, B, C, D, E, F)
where
    tarr![A::Size, B::Size, C::Size, D::Size, E::Size, F::Size,]:
        typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE;

    type Size =
        <tarr![A::Size, B::Size, C::Size, D::Size, E::Size, F::Size,] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: Parse<II>,
> Parse<II> for (A, B, C, D, E, F)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: ToOutput,
> ToOutput for (A, B, C, D, E, F, G)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
> ListHashes for (A, B, C, D, E, F, G)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
> Topological for (A, B, C, D, E, F, G)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged, E: Tagged, F: Tagged, G: Tagged> Tagged
    for (A, B, C, D, E, F, G)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
        ],
    );
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size> Size for (A, B, C, D, E, F, G)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: ToOutput,
> ToOutput for (A, B, C, D, E, F, G, H)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
        self.7.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G, H)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
    H: ListHashes,
> ListHashes for (A, B, C, D, E, F, G, H)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
        self.7.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
    H: Topological,
> Topological for (A, B, C, D, E, F, G, H)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
        self.7.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged, E: Tagged, F: Tagged, G: Tagged, H: Tagged> Tagged
    for (A, B, C, D, E, F, G, H)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
            &H::TAGS,
        ],
    );
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size> Size
    for (A, B, C, D, E, F, G, H)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize =
        A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE + H::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G, H)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G, H)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
    H: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G, H)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: ToOutput,
> ToOutput for (A, B, C, D, E, F, G, H, I)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
        self.7.to_output(output);
        self.8.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G, H, I)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
    H: ListHashes,
    I: ListHashes,
> ListHashes for (A, B, C, D, E, F, G, H, I)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
        self.7.list_hashes(f);
        self.8.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
    H: Topological,
    I: Topological,
> Topological for (A, B, C, D, E, F, G, H, I)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
        self.7.accept_points(visitor);
        self.8.accept_points(visitor);
    }
}

impl<
    A: Tagged,
    B: Tagged,
    C: Tagged,
    D: Tagged,
    E: Tagged,
    F: Tagged,
    G: Tagged,
    H: Tagged,
    I: Tagged,
> Tagged for (A, B, C, D, E, F, G, H, I)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
            &H::TAGS,
            &I::TAGS,
        ],
    );
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size, I: Size> Size
    for (A, B, C, D, E, F, G, H, I)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize =
        A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE + H::SIZE + I::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G, H, I)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G, H, I)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
    H: MaybeHasNiche,
    I: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G, H, I)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
        I::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: ToOutput,
> ToOutput for (A, B, C, D, E, F, G, H, I, J)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
        self.7.to_output(output);
        self.8.to_output(output);
        self.9.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G, H, I, J)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
    H: ListHashes,
    I: ListHashes,
    J: ListHashes,
> ListHashes for (A, B, C, D, E, F, G, H, I, J)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
        self.7.list_hashes(f);
        self.8.list_hashes(f);
        self.9.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
    H: Topological,
    I: Topological,
    J: Topological,
> Topological for (A, B, C, D, E, F, G, H, I, J)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
        self.7.accept_points(visitor);
        self.8.accept_points(visitor);
        self.9.accept_points(visitor);
    }
}

impl<
    A: Tagged,
    B: Tagged,
    C: Tagged,
    D: Tagged,
    E: Tagged,
    F: Tagged,
    G: Tagged,
    H: Tagged,
    I: Tagged,
    J: Tagged,
> Tagged for (A, B, C, D, E, F, G, H, I, J)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
            &H::TAGS,
            &I::TAGS,
            &J::TAGS,
        ],
    );
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size, I: Size, J: Size> Size
    for (A, B, C, D, E, F, G, H, I, J)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE
        + B::SIZE
        + C::SIZE
        + D::SIZE
        + E::SIZE
        + F::SIZE
        + G::SIZE
        + H::SIZE
        + I::SIZE
        + J::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G, H, I, J)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G, H, I, J)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
    H: MaybeHasNiche,
    I: MaybeHasNiche,
    J: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G, H, I, J)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
        I::MnArray,
        J::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: InlineOutput,
    K: ToOutput,
> ToOutput for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
        self.7.to_output(output);
        self.8.to_output(output);
        self.9.to_output(output);
        self.10.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: InlineOutput,
    K: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G, H, I, J, K)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
    H: ListHashes,
    I: ListHashes,
    J: ListHashes,
    K: ListHashes,
> ListHashes for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
        self.7.list_hashes(f);
        self.8.list_hashes(f);
        self.9.list_hashes(f);
        self.10.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
    H: Topological,
    I: Topological,
    J: Topological,
    K: Topological,
> Topological for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
        self.7.accept_points(visitor);
        self.8.accept_points(visitor);
        self.9.accept_points(visitor);
        self.10.accept_points(visitor);
    }
}

impl<
    A: Tagged,
    B: Tagged,
    C: Tagged,
    D: Tagged,
    E: Tagged,
    F: Tagged,
    G: Tagged,
    H: Tagged,
    I: Tagged,
    J: Tagged,
    K: Tagged,
> Tagged for (A, B, C, D, E, F, G, H, I, J, K)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
            &H::TAGS,
            &I::TAGS,
            &J::TAGS,
            &K::TAGS,
        ],
    );
}

impl<
    A: Size,
    B: Size,
    C: Size,
    D: Size,
    E: Size,
    F: Size,
    G: Size,
    H: Size,
    I: Size,
    J: Size,
    K: Size,
> Size for (A, B, C, D, E, F, G, H, I, J, K)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
        K::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE
        + B::SIZE
        + C::SIZE
        + D::SIZE
        + E::SIZE
        + F::SIZE
        + G::SIZE
        + H::SIZE
        + I::SIZE
        + J::SIZE
        + K::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
        K::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: ParseInline<II>,
    K: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: ParseInline<II>,
    K: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
    H: MaybeHasNiche,
    I: MaybeHasNiche,
    J: MaybeHasNiche,
    K: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G, H, I, J, K)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
        I::MnArray,
        J::MnArray,
        K::MnArray,
    ];
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: InlineOutput,
    K: InlineOutput,
    L: ToOutput,
> ToOutput for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
        self.5.to_output(output);
        self.6.to_output(output);
        self.7.to_output(output);
        self.8.to_output(output);
        self.9.to_output(output);
        self.10.to_output(output);
        self.11.to_output(output);
    }
}

impl<
    A: InlineOutput,
    B: InlineOutput,
    C: InlineOutput,
    D: InlineOutput,
    E: InlineOutput,
    F: InlineOutput,
    G: InlineOutput,
    H: InlineOutput,
    I: InlineOutput,
    J: InlineOutput,
    K: InlineOutput,
    L: InlineOutput,
> InlineOutput for (A, B, C, D, E, F, G, H, I, J, K, L)
{
}

impl<
    A: ListHashes,
    B: ListHashes,
    C: ListHashes,
    D: ListHashes,
    E: ListHashes,
    F: ListHashes,
    G: ListHashes,
    H: ListHashes,
    I: ListHashes,
    J: ListHashes,
    K: ListHashes,
    L: ListHashes,
> ListHashes for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn list_hashes(&self, f: &mut impl FnMut(Hash)) {
        self.0.list_hashes(f);
        self.1.list_hashes(f);
        self.2.list_hashes(f);
        self.3.list_hashes(f);
        self.4.list_hashes(f);
        self.5.list_hashes(f);
        self.6.list_hashes(f);
        self.7.list_hashes(f);
        self.8.list_hashes(f);
        self.9.list_hashes(f);
        self.10.list_hashes(f);
        self.11.list_hashes(f);
    }
}

impl<
    A: Topological,
    B: Topological,
    C: Topological,
    D: Topological,
    E: Topological,
    F: Topological,
    G: Topological,
    H: Topological,
    I: Topological,
    J: Topological,
    K: Topological,
    L: Topological,
> Topological for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
        self.6.accept_points(visitor);
        self.7.accept_points(visitor);
        self.8.accept_points(visitor);
        self.9.accept_points(visitor);
        self.10.accept_points(visitor);
        self.11.accept_points(visitor);
    }
}

impl<
    A: Tagged,
    B: Tagged,
    C: Tagged,
    D: Tagged,
    E: Tagged,
    F: Tagged,
    G: Tagged,
    H: Tagged,
    I: Tagged,
    J: Tagged,
    K: Tagged,
    L: Tagged,
> Tagged for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    const TAGS: Tags = Tags(
        &[],
        &[
            &A::TAGS,
            &B::TAGS,
            &C::TAGS,
            &D::TAGS,
            &E::TAGS,
            &F::TAGS,
            &G::TAGS,
            &H::TAGS,
            &I::TAGS,
            &J::TAGS,
            &K::TAGS,
            &L::TAGS,
        ],
    );
}

impl<
    A: Size,
    B: Size,
    C: Size,
    D: Size,
    E: Size,
    F: Size,
    G: Size,
    H: Size,
    I: Size,
    J: Size,
    K: Size,
    L: Size,
> Size for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
        K::Size,
        L::Size,
    ]: typenum::FoldAdd<Output: Unsigned>,
{
    const SIZE: usize = A::SIZE
        + B::SIZE
        + C::SIZE
        + D::SIZE
        + E::SIZE
        + F::SIZE
        + G::SIZE
        + H::SIZE
        + I::SIZE
        + J::SIZE
        + K::SIZE
        + L::SIZE;

    type Size = <tarr![
        A::Size,
        B::Size,
        C::Size,
        D::Size,
        E::Size,
        F::Size,
        G::Size,
        H::Size,
        I::Size,
        J::Size,
        K::Size,
        L::Size,
    ] as typenum::FoldAdd>::Output;
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: ParseInline<II>,
    K: ParseInline<II>,
    L: Parse<II>,
> Parse<II> for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn parse(mut input: II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<
    II: ParseInput,
    A: ParseInline<II>,
    B: ParseInline<II>,
    C: ParseInline<II>,
    D: ParseInline<II>,
    E: ParseInline<II>,
    F: ParseInline<II>,
    G: ParseInline<II>,
    H: ParseInline<II>,
    I: ParseInline<II>,
    J: ParseInline<II>,
    K: ParseInline<II>,
    L: ParseInline<II>,
> ParseInline<II> for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn parse_inline(input: &mut II) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<
    A: MaybeHasNiche,
    B: MaybeHasNiche,
    C: MaybeHasNiche,
    D: MaybeHasNiche,
    E: MaybeHasNiche,
    F: MaybeHasNiche,
    G: MaybeHasNiche,
    H: MaybeHasNiche,
    I: MaybeHasNiche,
    J: MaybeHasNiche,
    K: MaybeHasNiche,
    L: MaybeHasNiche,
> MaybeHasNiche for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    type MnArray = tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
        I::MnArray,
        J::MnArray,
        K::MnArray,
        L::MnArray,
    ];
}
