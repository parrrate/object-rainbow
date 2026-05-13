use std::sync::Arc;

use object_rainbow::{ListHashes, Tagged, Topological};

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

#[test]
fn abc() {
    let v = Vec::from(Prefix::default().with(b"a").with(b"bc"));
    assert_eq!(v, b"abc");
}
