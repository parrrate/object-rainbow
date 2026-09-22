use std::ops::{Deref, DerefMut};

use axum::response::{IntoResponse, Response};
use object_rainbow::ToOutput;

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
