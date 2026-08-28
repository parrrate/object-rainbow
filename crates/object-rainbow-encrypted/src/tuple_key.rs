#[derive(Debug, thiserror::Error)]
pub enum LayeredError<O, I> {
    #[error(transparent)]
    Outer(O),
    #[error(transparent)]
    Inner(I),
}
