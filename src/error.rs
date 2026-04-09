#[macro_export]
/// Construct [`Error::Parse`].
macro_rules! error_parse {
    ($($t:tt)*) => {
        $crate::Error::Parse($crate::anyhow!($($t)*))
    };
}

#[macro_export]
/// Construct [`Error::Fetch`].
macro_rules! error_fetch {
    ($($t:tt)*) => {
        $crate::Error::Fetch($crate::anyhow!($($t)*))
    };
}

/// Errors encountered during fetching an object. Mostly related to parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Arbitrary parsing error.
    #[error(transparent)]
    Parse(anyhow::Error),
    /// Arbitrary fetching error.
    #[error(transparent)]
    Fetch(anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Data left after an [`Inline`] got parsed.
    #[error("extra input left")]
    ExtraInputLeft,
    /// EOF.
    #[error("end of input")]
    EndOfInput,
    /// Overran [`PointInput`]'s [`Address`] vector.
    #[error("address index out of bounds")]
    AddressOutOfBounds,
    /// [`Address::hash`] doesn't match what [`Resolve`] returned.
    #[error("hash resolution mismatch")]
    ResolutionMismatch,
    /// [`FullHash::full_hash`] doesn't match [`Singular::hash`].
    #[error("data hash mismatch")]
    DataMismatch,
    /// Discriminant out of range for an [`Enum`].
    #[error("discriminant overflow")]
    DiscriminantOverflow,
    /// Unepxected zero for a non-zero value.
    #[error("zero")]
    Zero,
    /// Value out of bounds for a certain type.
    #[error("out of bounds")]
    OutOfBounds,
    /// Current architecture (32-bit) is unable to handle lengths of this size.
    #[error("length out of bounds")]
    UnsupportedLength,
    /// Not UTF-8.
    #[error(transparent)]
    Utf8(std::string::FromUtf8Error),
    /// [`Resolve::extension`] (or related things) were unable to resolve the extension.
    #[error("unknown extension")]
    UnknownExtension,
    /// Extension type didn't match what we asked for. This might be turned into panic later.
    #[error("wrong extension type")]
    ExtensionType,
    #[error("not implemented")]
    Unimplemented,
}

impl Error {
    /// Construct [`Error::Parse`] from another error.
    pub fn parse(e: impl Into<anyhow::Error>) -> Self {
        Self::Parse(e.into())
    }

    /// Construct [`Error::Fetch`] from another error.
    pub fn fetch(e: impl Into<anyhow::Error>) -> Self {
        Self::Fetch(e.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
