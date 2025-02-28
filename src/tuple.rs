use super::*;

impl<A: ToOutput, B: ToOutput> ToOutput for (A, B) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
    }
}

impl<A: Inline, B: Object> Object for (A, B) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
    }

    fn parse(mut input: Input) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse()?))
    }

    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS]);
}

impl<A: Inline, B: Inline> Inline for (A, B) {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?))
    }
}

impl<A: ReflessInline, B: ReflessObject> ReflessObject for (A, B) {
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse()?))
    }
}

impl<A: ReflessInline, B: ReflessInline> ReflessInline for (A, B) {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?))
    }
}

impl<A: Size, B: Size> Size for (A, B) {
    const SIZE: usize = A::SIZE + B::SIZE;
}

impl<A: ToOutput, B: ToOutput, C: ToOutput> ToOutput for (A, B, C) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
    }
}

impl<A: Inline, B: Inline, C: Object> Object for (A, B, C) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
    }

    fn parse(mut input: Input) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?, input.parse()?))
    }

    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS]);
}

impl<A: Inline, B: Inline, C: Inline> Inline for (A, B, C) {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessObject> ReflessObject for (A, B, C) {
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
        Ok((input.parse_inline()?, input.parse_inline()?, input.parse()?))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline> ReflessInline for (A, B, C) {
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: Size, B: Size, C: Size> Size for (A, B, C) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE;
}

impl<A: ToOutput, B: ToOutput, C: ToOutput, D: ToOutput> ToOutput for (A, B, C, D) {
    fn to_output(&self, output: &mut dyn Output) {
        self.0.to_output(output);
        self.1.to_output(output);
        self.2.to_output(output);
        self.3.to_output(output);
    }
}

impl<A: Inline, B: Inline, C: Inline, D: Object> Object for (A, B, C, D) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
    }

    fn parse(mut input: Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }

    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS]);
}

impl<A: Inline, B: Inline, C: Inline, D: Inline> Inline for (A, B, C, D) {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessObject> ReflessObject
    for (A, B, C, D)
{
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline> ReflessInline
    for (A, B, C, D)
{
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: Size, B: Size, C: Size, D: Size> Size for (A, B, C, D) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE;
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Object> Object for (A, B, C, D, E) {
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
    }

    fn parse(mut input: Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }

    const TAGS: Tags = Tags(&[], &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS, &E::TAGS]);
}

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline> Inline for (A, B, C, D, E) {
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline, E: ReflessObject>
    ReflessObject for (A, B, C, D, E)
{
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }
}

impl<A: ReflessInline, B: ReflessInline, C: ReflessInline, D: ReflessInline, E: ReflessInline>
    ReflessInline for (A, B, C, D, E)
{
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
        ))
    }
}

impl<A: Size, B: Size, C: Size, D: Size, E: Size> Size for (A, B, C, D, E) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE;
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Object> Object
    for (A, B, C, D, E, F)
{
    fn accept_points(&self, visitor: &mut impl PointVisitor) {
        self.0.accept_points(visitor);
        self.1.accept_points(visitor);
        self.2.accept_points(visitor);
        self.3.accept_points(visitor);
        self.4.accept_points(visitor);
        self.5.accept_points(visitor);
    }

    fn parse(mut input: Input) -> crate::Result<Self> {
        Ok((
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse_inline()?,
            input.parse()?,
        ))
    }

    const TAGS: Tags = Tags(
        &[],
        &[&A::TAGS, &B::TAGS, &C::TAGS, &D::TAGS, &E::TAGS, &F::TAGS],
    );
}

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Inline> Inline
    for (A, B, C, D, E, F)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessObject,
> ReflessObject for (A, B, C, D, E, F)
{
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
> ReflessInline for (A, B, C, D, E, F)
{
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size> Size for (A, B, C, D, E, F) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE;
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Inline, G: Object> Object
    for (A, B, C, D, E, F, G)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Inline, G: Inline> Inline
    for (A, B, C, D, E, F, G)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessObject,
> ReflessObject for (A, B, C, D, E, F, G)
{
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    A: ReflessInline,
    B: ReflessInline,
    C: ReflessInline,
    D: ReflessInline,
    E: ReflessInline,
    F: ReflessInline,
    G: ReflessInline,
> ReflessInline for (A, B, C, D, E, F, G)
{
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size> Size for (A, B, C, D, E, F, G) {
    const SIZE: usize = A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE;
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Inline, G: Inline, H: Object> Object
    for (A, B, C, D, E, F, G, H)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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

impl<A: Inline, B: Inline, C: Inline, D: Inline, E: Inline, F: Inline, G: Inline, H: Inline> Inline
    for (A, B, C, D, E, F, G, H)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size> Size
    for (A, B, C, D, E, F, G, H)
{
    const SIZE: usize =
        A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE + H::SIZE;
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Object,
> Object for (A, B, C, D, E, F, G, H, I)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
> Inline for (A, B, C, D, E, F, G, H, I)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size, I: Size> Size
    for (A, B, C, D, E, F, G, H, I)
{
    const SIZE: usize =
        A::SIZE + B::SIZE + C::SIZE + D::SIZE + E::SIZE + F::SIZE + G::SIZE + H::SIZE + I::SIZE;
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Object,
> Object for (A, B, C, D, E, F, G, H, I, J)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Inline,
> Inline for (A, B, C, D, E, F, G, H, I, J)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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

impl<A: Size, B: Size, C: Size, D: Size, E: Size, F: Size, G: Size, H: Size, I: Size, J: Size> Size
    for (A, B, C, D, E, F, G, H, I, J)
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Inline,
    K: Object,
> Object for (A, B, C, D, E, F, G, H, I, J, K)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Inline,
    K: Inline,
> Inline for (A, B, C, D, E, F, G, H, I, J, K)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Inline,
    K: Inline,
    L: Object,
> Object for (A, B, C, D, E, F, G, H, I, J, K, L)
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

    fn parse(mut input: Input) -> crate::Result<Self> {
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
    A: Inline,
    B: Inline,
    C: Inline,
    D: Inline,
    E: Inline,
    F: Inline,
    G: Inline,
    H: Inline,
    I: Inline,
    J: Inline,
    K: Inline,
    L: Inline,
> Inline for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    fn parse_inline(input: &mut Input) -> crate::Result<Self> {
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
    fn parse(mut input: ReflessInput) -> crate::Result<Self> {
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
    fn parse_inline(input: &mut ReflessInput) -> crate::Result<Self> {
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
}
