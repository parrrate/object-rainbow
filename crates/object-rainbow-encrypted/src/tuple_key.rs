#[derive(Debug, thiserror::Error)]
pub enum LayeredError<O> {
    #[error(transparent)]
    Outer(O),
}
