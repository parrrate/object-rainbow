use crate::*;

impl ToOutput for char {
    fn to_output(&self, output: &mut dyn Output) {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf).to_output(output);
    }
}

impl<I: ParseInput> Parse<I> for char {
    fn parse(input: I) -> crate::Result<Self> {
        Self::parse_as_inline(input)
    }
}

impl<I: ParseInput> ParseInline<I> for char {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let c0 = input.parse_inline::<u8>()?;
        if c0 & 0b11000000 != 0b11000000 {
            let c = str::from_utf8(&[c0])
                .map_err(crate::Error::parse)?
                .chars()
                .next()
                .unwrap();
            return Ok(c);
        }
        let c1 = input.parse_inline::<u8>()?;
        if c0 & 0b00100000 == 0 {
            let c = str::from_utf8(&[c0, c1])
                .map_err(crate::Error::parse)?
                .chars()
                .next()
                .unwrap();
            return Ok(c);
        }
        let c2 = input.parse_inline::<u8>()?;
        if c0 & 0b00010000 == 0 {
            let c = str::from_utf8(&[c0, c1, c2])
                .map_err(crate::Error::parse)?
                .chars()
                .next()
                .unwrap();
            return Ok(c);
        }
        let c3 = input.parse_inline::<u8>()?;
        let c = str::from_utf8(&[c0, c1, c2, c3])
            .map_err(crate::Error::parse)?
            .chars()
            .next()
            .unwrap();
        Ok(c)
    }
}

impl InlineOutput for char {}
impl Tagged for char {}
impl ListHashes for char {}
impl Topological for char {}

#[test]
fn reparse() -> crate::Result<()> {
    assert_eq!('x'.reparse()?, 'x');
    assert_eq!('ч'.reparse()?, 'ч');
    Ok(())
}
