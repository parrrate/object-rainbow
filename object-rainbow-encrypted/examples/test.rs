use chacha20poly1305::{ChaCha20Poly1305, aead::Aead};
use object_rainbow::{Fetch, Object};
use object_rainbow_encrypted::{Key, encrypt_point};
use object_rainbow_fetchall::fetchall;
use object_rainbow_point::{IntoPoint, Point};
use sha2::digest::generic_array::GenericArray;

#[derive(Debug, Clone, Copy)]
struct Test([u8; 32]);

impl Key for Test {
    type Error = chacha20poly1305::Error;

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        println!("encrypt");
        let cipher = {
            use chacha20poly1305::KeyInit;
            ChaCha20Poly1305::new(&self.0.into())
        };
        let nonce = &{
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize()
        };
        let nonce = &nonce.as_slice()[..12];
        let encrypted = cipher
            .encrypt(GenericArray::from_slice(nonce), data)
            .expect("we do not handle decryption errors");
        [nonce, encrypted.as_slice()].concat()
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
        let cipher = {
            use chacha20poly1305::KeyInit;
            ChaCha20Poly1305::new(&self.0.into())
        };
        cipher.decrypt(GenericArray::from_slice(&data[..12]), &data[12..])
    }
}

async fn iterate<T: Object<Extra>, Extra: 'static + Send + Sync + Clone>(
    point: Point<T>,
    extra: Extra,
) -> anyhow::Result<Point<T>> {
    let map = fetchall(&point.fetch().await?).await?;
    let resolve = map.to_resolve();
    Ok(point.with_resolve(resolve, extra))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting");
    smol::block_on(async move {
        let point = (
            (*b"alisa", *b"feistel").point().point(),
            [1u8, 2, 3, 4].point(),
        )
            .point();
        let key = Test(std::array::from_fn(|i| i as _));
        let point = encrypt_point(key, point).await?;
        println!("after encryption 1");
        let point = iterate(point, key).await?;
        let point = point.fetch().await?.into_inner().point();
        let point = encrypt_point(key, point).await?;
        println!("after encryption 2");
        let point = point.fetch().await?.into_inner().point();
        assert_eq!(
            point.fetch().await?.0.fetch().await?.fetch().await?.0,
            *b"alisa",
        );
        assert_eq!(
            point.fetch().await?.0.fetch().await?.fetch().await?.1,
            *b"feistel",
        );
        println!("all right");
        Ok(())
    })
}
