//! Original: <https://docs.rs/sha2-const/0.1.2/src/sha2_const/lib.rs.html>
//!
//! Original license: MIT OR Apache-2.0, copyright Saleem Rashid

mod constants;
mod sha;
mod util;

use self::constants::H256;
use self::util::memcpy;

macro_rules! sha {
    (
        $(#[$doc:meta])* $name:ident,
        $size:literal,
        $inner:ty,
        $iv:ident
    ) => {
        $(#[$doc])*
        #[derive(Clone)]
        pub struct $name {
            inner: $inner,
        }

        impl $name {
            pub const DIGEST_SIZE: usize = $size;

            pub const fn new() -> Self {
                Self {
                    inner: <$inner>::new($iv),
                }
            }

            #[must_use]
            pub const fn update(mut self, input: &[u8]) -> Self {
                self.inner.update(&input);
                self
            }

            #[must_use]
            pub const fn finalize(self) -> [u8; Self::DIGEST_SIZE] {
                let digest = self.inner.finalize();
                let mut truncated = [0; Self::DIGEST_SIZE];
                memcpy(&mut truncated, 0, &digest, 0, Self::DIGEST_SIZE);
                truncated
            }
        }
    };
}

sha!(Sha256, 32, sha::Sha256, H256);
