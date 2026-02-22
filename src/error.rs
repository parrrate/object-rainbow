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

#[macro_export]
/// Construct [`Error::Operation`].
macro_rules! error_operation {
    ($($t:tt)*) => {
        $crate::Error::Operation($crate::anyhow!($($t)*))
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
    /// Arbitrary operation error.
    #[error(transparent)]
    Operation(anyhow::Error),
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
    #[error("full hash mismatch")]
    FullHashMismatch,
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
    Utf8(#[from] std::string::FromUtf8Error),
    /// [`Resolve::extension`] (or related things) were unable to resolve the extension.
    #[error("unknown extension")]
    UnknownExtension,
    /// Extension type didn't match what we asked for. This might be turned into panic later.
    #[error("wrong extension type")]
    ExtensionType,
    #[error("not implemented")]
    Unimplemented,
    #[error("hash not found in the resolve")]
    HashNotFound,
    #[error("interrupted")]
    Interrupted,
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

    /// Construct [`Error::Operation`] from another error.
    pub fn operation(e: impl Into<anyhow::Error>) -> Self {
        Self::Fetch(e.into())
    }

    pub fn io(e: impl Into<std::io::Error>) -> Self {
        e.into().into()
    }

    fn io_kind(&self) -> std::io::ErrorKind {
        use std::io::ErrorKind;
        match self {
            Error::Parse(_) => ErrorKind::InvalidData,
            Error::Fetch(_) => ErrorKind::Other,
            Error::Operation(_) => ErrorKind::Other,
            Error::Io(e) => e.kind(),
            Error::ExtraInputLeft => ErrorKind::InvalidData,
            Error::EndOfInput => ErrorKind::UnexpectedEof,
            Error::AddressOutOfBounds => ErrorKind::Other,
            Error::ResolutionMismatch => ErrorKind::InvalidData,
            Error::FullHashMismatch => ErrorKind::InvalidData,
            Error::DiscriminantOverflow => ErrorKind::InvalidData,
            Error::Zero => ErrorKind::InvalidData,
            Error::OutOfBounds => ErrorKind::InvalidData,
            Error::UnsupportedLength => ErrorKind::FileTooLarge,
            Error::Utf8(_) => ErrorKind::InvalidData,
            Error::UnknownExtension => ErrorKind::Other,
            Error::ExtensionType => ErrorKind::Other,
            Error::Unimplemented => ErrorKind::Unsupported,
            Error::HashNotFound => ErrorKind::NotFound,
            Error::Interrupted => ErrorKind::Interrupted,
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Io(e) => e,
            e => Self::new(e.io_kind(), e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
