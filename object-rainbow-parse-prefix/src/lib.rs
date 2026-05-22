use std::sync::Arc;

use object_rainbow::{
    InlineOutput, ListHashes, Output, Parse, ParseInline, PointInput, Tagged, ToOutput,
    Topological,
    length_prefixed::LpBytes,
    map_extra::{MapExtra, SmExtra, StaticMap},
};

#[derive(Debug, Clone, Default)]
pub struct Prefix(Option<Arc<(Vec<u8>, Self)>>);

impl PartialEq for Prefix {
    fn eq(&self, other: &Self) -> bool {
        Vec::from(self.clone()) == Vec::from(other.clone())
    }
}

impl Eq for Prefix {}

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

    pub fn pop_n(&mut self, mut n: usize) -> object_rainbow::Result<()> {
        if n > self.len() {
            return Err(object_rainbow::error_operation!(
                "Prefix isn't at least {n} bytes long"
            ));
        }
        while n > 0
            && let Some((mut v, rest)) = self.0.take().map(Arc::unwrap_or_clone)
        {
            if n >= v.len() {
                n -= v.len();
                *self = rest;
            } else {
                v.drain(v.len() - n..);
                n = 0;
                self.0 = Some((v, rest).into());
            }
        }
        assert_eq!(n, 0);
        Ok(())
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

impl From<&[u8]> for Prefix {
    fn from(slice: &[u8]) -> Self {
        Self::default().with(slice)
    }
}

#[derive(Debug, Tagged, ListHashes, Topological, Clone)]
pub struct WithPrefix<T> {
    prefix: Prefix,
    value: T,
}

impl<T: PartialEq> PartialEq for WithPrefix<T> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix.len() == other.prefix.len() && self.value == other.value
    }
}

impl<T: Eq> Eq for WithPrefix<T> {}

impl<T> WithPrefix<T> {
    pub fn new(prefix: Prefix, value: T) -> object_rainbow::Result<Self>
    where
        T: ToOutput,
    {
        if value.vec().starts_with(&Vec::from(prefix.clone())) {
            Ok(Self { prefix, value })
        } else {
            Err(object_rainbow::error_consistency!(
                "`value` doesn't start with `prefix`",
            ))
        }
    }

    pub fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn pop_n(&mut self, n: usize) -> object_rainbow::Result<()> {
        self.prefix.pop_n(n)
    }
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

impl<T: InlineOutput> InlineOutput for WithPrefix<T> {}

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

pub struct StaticWithByte;

impl<E> StaticMap<(u8, (Prefix, E))> for StaticWithByte {
    type Mapped = (Prefix, E);

    fn map_extra((byte, (prefix, e)): (u8, (Prefix, E))) -> Self::Mapped {
        (prefix.with(vec![byte]), e)
    }
}

pub type WithByte = SmExtra<StaticWithByte>;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Default,
)]
pub struct WithBytes(pub LpBytes);

impl<E: 'static + Clone> MapExtra<(Prefix, E)> for WithBytes {
    type Mapped = (Prefix, E);

    fn map_extra(&self, (prefix, e): (Prefix, E)) -> Self::Mapped {
        (prefix.with(self.0.0.clone()), e)
    }
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
)]
pub struct PrefixRoot;

impl<E: 'static + Clone> MapExtra<E> for PrefixRoot {
    type Mapped = (Prefix, E);

    fn map_extra(&self, e: E) -> Self::Mapped {
        (Prefix::default(), e)
    }
}

#[test]
fn abc() {
    let v = Vec::from(Prefix::default().with(b"a").with(b"bc"));
    assert_eq!(v, b"abc");
}

#[test]
fn parse_then_output() -> object_rainbow::Result<()> {
    use object_rainbow::ParseSliceExtra;
    let data = WithPrefix::<Vec<u8>>::parse_slice_extra(
        b"cd",
        &(Arc::new(Vec::new()) as _),
        &(Prefix::default().with(b"a").with(b"b"), ()),
    )?;
    assert_eq!(data.value, b"abcd");
    assert_eq!(data.vec(), b"cd");
    Ok(())
}
