use std::{fmt::Display, str::FromStr};

use crate::{with_repr::WithRepr, *};

/// Zero-terminated value. Used to make [`Inline`]s out of [`Object`]s which don't contain zeroes.
///
/// If you can't guarantee absence of zeroes, see [`length_prefixed::Lp`].
#[derive(Debug, Tagged, ListHashes, Topological, ParseAsInline)]
pub struct Zt<T> {
    inner: Arc<WithRepr<T>>,
}

impl<T> PartialEq for Zt<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Zt<T> {}

impl<T: ByteOrd> PartialOrd for Zt<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ByteOrd> Ord for Zt<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T: ToOutput> Zt<T> {
    /// Create a zero-terminated value.
    ///
    /// Pre-computes the output, errors if it contains a zero.
    pub fn new(object: T) -> crate::Result<Self> {
        let inner = WithRepr::new(object);
        if inner.data().contains(&0) {
            Err(Error::Zero)
        } else {
            Ok(Self {
                inner: Arc::new(inner),
            })
        }
    }
}

impl<T> Clone for Zt<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for Zt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.object()
    }
}

impl<T: ToOutput> ToOutput for Zt<T> {
    fn to_output(&self, output: &mut impl Output) {
        self.inner.to_output(output);
        if output.is_real() {
            output.write(&[0]);
        }
    }
}

impl<T: ToOutput> InlineOutput for Zt<T> {}

impl<T: Parse<I>, I: ParseInput> ParseInline<I> for Zt<T> {
    fn parse_inline(input: &mut I) -> crate::Result<Self> {
        let inner = Arc::new(WithRepr::parse_zero_terminated(input)?);
        Ok(Self { inner })
    }
}

impl<T: MaybeHasNiche<MnArray = NoNiche<NicheForUnsized>>> MaybeHasNiche for Zt<T> {
    type MnArray = NoNiche<NicheForUnsized>;
}

impl<T: FromStr<Err: 'static + Send + Sync + std::error::Error> + ToOutput> FromStr for Zt<T> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s.parse().map_err(Error::parse)?)
    }
}

impl<T: AsRef<str>> AsRef<str> for Zt<T> {
    fn as_ref(&self) -> &str {
        self.inner.object().as_ref()
    }
}

impl<T: Display> Display for Zt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.object().fmt(f)
    }
}
