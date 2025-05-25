pub trait Key {
    fn encrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
    fn decrypt(&self, data: &[u8]) -> object_rainbow::Result<Vec<u8>>;
}

struct _Encrypted<K, T> {
    key: K,
    decrypted: T,
}
