use typenum::tarr;

use crate::*;

impl<A: ToOutput, B: ToOutput> ToOutput for (A, B) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl<Extra: 'static, A: Topological<Extra>, B: Topological<Extra>> Topological<Extra> for (A, B) {
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged> Tagged for (A, B) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS]);
}

impl<Extra: 'static, A: Inline<Extra>, B: Object<Extra>> Object<Extra> for (A, B) {}

impl<Extra: 'static, A: Inline<Extra>, B: Inline<Extra>> Inline<Extra> for (A, B) {}

impl<A: ReflessInline, B: ReflessObject> ReflessObject for (A, B) {}

impl<A: ReflessInline, B: ReflessInline> ReflessInline for (A, B) {}

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

impl<A: MaybeHasNiche, B: MaybeHasNiche> MaybeHasNiche for (A, B)
where
    Self: Size,
    tarr![A::MnArray, B::MnArray,]: MnArray,
{
    type MnArray = tarr![A::MnArray, B::MnArray,];
}

impl<A: ToOutput, B: ToOutput, C: ToOutput> ToOutput for (A, B, C) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
    }
}

impl<Extra: 'static, A: Topological<Extra>, B: Topological<Extra>, C: Topological<Extra>>
    Topological<Extra> for (A, B, C)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged> Tagged for (A, B, C) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS]);
}

impl<Extra: 'static, A: Inline<Extra>, B: Inline<Extra>, C: Object<Extra>> Object<Extra>
    for (A, B, C)
{
}

impl<Extra: 'static, A: Inline<Extra>, B: Inline<Extra>, C: Inline<Extra>> Inline<Extra>
    for (A, B, C)
{
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessObject> ReflessObject for (A, B, C) {}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline> ReflessInline for (A, B, C) {}

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

impl<A: MaybeHasNiche, B: MaybeHasNiche, C: MaybeHasNiche> MaybeHasNiche for (A, B, C)
where
    Self: Size,
    tarr![A::MnArray, B::MnArray, C::MnArray,]: MnArray,
{
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray,];
}

impl<A: ToOutput, B: ToOutput, C: ToOutput, D: ToOutput> ToOutput for (A, B, C, D) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
    }
}

impl<
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
> Topological<Extra> for (A, B, C, D)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
    }
}

impl<A: Tagged, B: Tagged, C: Tagged, D: Tagged> Tagged for (A, B, C, D) {
    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS]);
}

impl<Extra: 'static, A: Inline<Extra>, B: Inline<Extra>, C: Inline<Extra>, D: Object<Extra>>
    Object<Extra> for (A, B, C, D)
{
}

impl<Extra: 'static, A: Inline<Extra>, B: Inline<Extra>, C: Inline<Extra>, D: Inline<Extra>>
    Inline<Extra> for (A, B, C, D)
{
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessObject> ReflessObject
    for (A, B, C, D)
{
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline> ReflessInline
    for (A, B, C, D)
{
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
where
    Self: Size,
    tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray,]: MnArray,
{
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray,];
}

impl<A: ToOutput, B: ToOutput, C: ToOutput, D: ToOutput, E: ToOutput> ToOutput for (A, B, C, D, E) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
        self.4.to_output(output);
    }
}

impl<
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Object<Extra>,
> Object<Extra> for (A, B, C, D, E)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E)
{
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline, E: ReflessObject>
    ReflessObject for (A, B, C, D, E)
{
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline, E: ReflessInline>
    ReflessInline for (A, B, C, D, E)
{
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
where
    Self: Size,
    tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray, E::MnArray,]: MnArray,
{
    type MnArray = tarr![A::MnArray, B::MnArray, C::MnArray, D::MnArray, E::MnArray,];
}

impl<A: ToOutput, B: ToOutput, C: ToOutput, D: ToOutput, E: ToOutput, F: ToOutput> ToOutput
    for (A, B, C, D, E, F)
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessObject,
> ReflessObject for (A, B, C, D, E, F)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
> ReflessInline for (A, B, C, D, E, F)
{
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
where
    Self: Size,
    tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
    ]: MnArray,
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

impl<A: ToOutput, B: ToOutput, C: ToOutput, D: ToOutput, E: ToOutput, F: ToOutput, G: ToOutput>
    ToOutput for (A, B, C, D, E, F, G)
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G)
{
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
where
    Self: Size,
    tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
    ]: MnArray,
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
    A: ToOutput,
    B: ToOutput,
    C: ToOutput,
    D: ToOutput,
    E: ToOutput,
    F: ToOutput,
    G: ToOutput,
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
    H: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G, H)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G, H)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G, H)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G, H)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G, H)
{
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
where
    Self: Size,
    tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
    ]: MnArray,
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
    A: ToOutput,
    B: ToOutput,
    C: ToOutput,
    D: ToOutput,
    E: ToOutput,
    F: ToOutput,
    G: ToOutput,
    H: ToOutput,
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
    H: Topological<Extra>,
    I: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G, H, I)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G, H, I)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G, H, I)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G, H, I)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G, H, I)
{
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
where
    Self: Size,
    tarr![
        A::MnArray,
        B::MnArray,
        C::MnArray,
        D::MnArray,
        E::MnArray,
        F::MnArray,
        G::MnArray,
        H::MnArray,
        I::MnArray,
    ]: MnArray,
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
    A: ToOutput,
    B: ToOutput,
    C: ToOutput,
    D: ToOutput,
    E: ToOutput,
    F: ToOutput,
    G: ToOutput,
    H: ToOutput,
    I: ToOutput,
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
    H: Topological<Extra>,
    I: Topological<Extra>,
    J: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G, H, I, J)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G, H, I, J)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G, H, I, J)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G, H, I, J)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G, H, I, J)
{
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
where
    Self: Size,
    tarr![
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
    ]: MnArray,
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
    A: ToOutput,
    B: ToOutput,
    C: ToOutput,
    D: ToOutput,
    E: ToOutput,
    F: ToOutput,
    G: ToOutput,
    H: ToOutput,
    I: ToOutput,
    J: ToOutput,
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
    H: Topological<Extra>,
    I: Topological<Extra>,
    J: Topological<Extra>,
    K: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Inline<Extra>,
    K: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G, H, I, J, K)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Inline<Extra>,
    K: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G, H, I, J, K)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessInline,
    K: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G, H, I, J, K)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessInline,
    K: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G, H, I, J, K)
{
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
where
    Self: Size,
    tarr![
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
    ]: MnArray,
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
    A: ToOutput,
    B: ToOutput,
    C: ToOutput,
    D: ToOutput,
    E: ToOutput,
    F: ToOutput,
    G: ToOutput,
    H: ToOutput,
    I: ToOutput,
    J: ToOutput,
    K: ToOutput,
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
    Extra: 'static,
    A: Topological<Extra>,
    B: Topological<Extra>,
    C: Topological<Extra>,
    D: Topological<Extra>,
    E: Topological<Extra>,
    F: Topological<Extra>,
    G: Topological<Extra>,
    H: Topological<Extra>,
    I: Topological<Extra>,
    J: Topological<Extra>,
    K: Topological<Extra>,
    L: Topological<Extra>,
> Topological<Extra> for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor<Extra>) {
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
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Inline<Extra>,
    K: Inline<Extra>,
    L: Object<Extra>,
> Object<Extra> for (A, B, C, D, E, F, G, H, I, J, K, L)
{
}

impl<
    Extra: 'static,
    A: Inline<Extra>,
    B: Inline<Extra>,
    C: Inline<Extra>,
    D: Inline<Extra>,
    E: Inline<Extra>,
    F: Inline<Extra>,
    G: Inline<Extra>,
    H: Inline<Extra>,
    I: Inline<Extra>,
    J: Inline<Extra>,
    K: Inline<Extra>,
    L: Inline<Extra>,
> Inline<Extra> for (A, B, C, D, E, F, G, H, I, J, K, L)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessInline,
    K: ReflessInline,
    L: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G, H, I, J, K, L)
{
}

impl<
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
    H: ReflessInline,
    I: ReflessInline,
    J: ReflessInline,
    K: ReflessInline,
    L: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G, H, I, J, K, L)
{
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
where
    Self: Size,
    tarr![
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
    ]: MnArray,
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
