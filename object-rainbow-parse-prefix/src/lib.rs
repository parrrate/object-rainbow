use std::sync::Arc;

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
}
