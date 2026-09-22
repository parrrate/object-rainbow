use std::ops::{Deref, DerefMut};

use axum::{
    body::Bytes,
    extract::{FromRequest, Request, rejection::BytesRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use object_rainbow::{ParseSliceRefless, ReflessObject, ToOutput};

#[derive(Debug, Clone, Copy)]
pub struct Refless<T>(pub T);

impl<T> Deref for Refless<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Refless<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: ToOutput> IntoResponse for Refless<T> {
    fn into_response(self) -> Response {
        self.vec().into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReflessRejection {
    #[error(transparent)]
    Rainbow(#[from] object_rainbow::Error),
    #[error(transparent)]
    Bytes(#[from] BytesRejection),
}

impl IntoResponse for ReflessRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Rainbow(error) => {
                (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()).into_response()
            }
            Self::Bytes(bytes) => bytes.into_response(),
        }
    }
}

impl<T: ReflessObject, S: Send + Sync> FromRequest<S> for Refless<T> {
    type Rejection = ReflessRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let object = T::parse_slice_refless(&Bytes::from_request(req, state).await?)?;
        Ok(Self(object))
    }
}
