use std::sync::Arc;

use object_rainbow::{
    ListHashes, Output, Parse, ParseInline, PointInput, Tagged, ToOutput, Topological,
};

#[derive(Clone, Default)]
pub struct Prefix(Option<Arc<(Vec<u8>, Self)>>);

impl Prefix {
    pub fn len(&self) -> usize {
        let mut total = 0;
        let mut this = self;
        while let Some((v, rest)) = this.0.as_deref() {
            total += v.len();
            this = rest;
        }
        total
    }

    pub fn is_empty(&self) -> bool {
        let mut this = self;
        while let Some((v, rest)) = this.0.as_deref() {
            if !v.is_empty() {
                return false;
            }
            this = rest;
        }
        true
    }

    pub fn with(&self, suffix: impl Into<Vec<u8>>) -> Self {
        Self(Some(Arc::new((suffix.into(), self.clone()))))
    }

    fn write_to(&self, mut dest: &mut [u8]) {
        let mut this = self;
        while let Some((v, rest)) = this.0.as_deref() {
            let part;
            (dest, part) = dest.split_at_mut(dest.len() - v.len());
            part.copy_from_slice(v);
            this = rest;
        }
        assert!(dest.is_empty());
    }
}

impl Tagged for Prefix {}
impl ListHashes for Prefix {}
impl Topological for Prefix {}

impl From<Prefix> for Vec<u8> {
    fn from(prefix: Prefix) -> Self {
        let mut vec = vec![0; prefix.len()];
        prefix.write_to(&mut vec);
        vec
    }
}

#[derive(Tagged, ListHashes, Topological, Clone)]
pub struct WithPrefix<T> {
    prefix: Prefix,
    value: T,
}

struct PrefixOutput<'a, O> {
    len: usize,
    output: &'a mut O,
}

impl<O: Output> Output for PrefixOutput<'_, O> {
    fn write(&mut self, data: &[u8]) {
        if self.output.is_real() {
            let n = self.len.min(data.len());
            self.len -= n;
            self.output.write(&data[n..]);
        }
        if self.output.is_mangling() {
            self.output.write(data);
        }
    }

    fn is_mangling(&self) -> bool {
        self.output.is_mangling()
    }

    fn is_real(&self) -> bool {
        self.output.is_real()
    }
}

impl<T: ToOutput> ToOutput for WithPrefix<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.value.to_output(&mut PrefixOutput {
            len: self.prefix.len(),
            output,
        });
    }
}

impl<
    T: Parse<J>,
    I: PointInput<Extra = (Prefix, E), WithExtra<E> = J>,
    J: PointInput<Extra = E>,
    E: 'static + Send + Sync + Clone,
> Parse<I> for WithPrefix<T>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let prefix = input.extra().0.clone();
        input.push_front(prefix.clone())?;
        let value = input.map_extra(|(_, e)| e).parse()?;
        Ok(Self { prefix, value })
    }
}

impl<
    T: ParseInline<J>,
    I: PointInput<Extra = (Prefix, E), WithExtra<E> = J>,
    J: PointInput<Extra = E>,
    E: 'static + Send + Sync + Clone,
> ParseInline<I> for WithPrefix<T>
{
    fn parse_inline(input: &mut I) -> object_rainbow::Result<Self> {
        let prefix = input.extra().0.clone();
        input.push_front(prefix.clone())?;
        let value = input.parse_inline_extra(input.extra().1.clone())?;
        Ok(Self { prefix, value })
    }
}

#[test]
fn abc() {
    let v = Vec::from(Prefix::default().with(b"a").with(b"bc"));
    assert_eq!(v, b"abc");
}

#[test]
fn parse_extra() -> object_rainbow::Result<()> {
    use object_rainbow::ParseSliceExtra;
    let data = WithPrefix::<Vec<u8>>::parse_slice_extra(
        b"cd",
        &(Arc::new(Vec::new()) as _),
        &(Prefix::default().with(b"a").with(b"b"), ()),
    )?;
    assert_eq!(data.value, b"abcd");
    Ok(())
}
