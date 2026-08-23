use std::pin::pin;

use fastcdc::v2020::{AsyncStreamCDC, Normalization};
use futures_util::{AsyncRead, Stream, StreamExt, TryStreamExt};
use genawaiter_try_stream::try_stream;
use object_rainbow::{DiffHashes, Fetch, Hash, InlineOutput, Singular, SizeExt, ToOutput};
use object_rainbow_point::{IntoPoint, Point};
use sha2::{Digest, Sha256};
use static_assertions::const_assert_eq;

#[derive(ToOutput)]
pub struct Chunks {
    chunks: Vec<Chunk>,
}

impl Chunks {
    pub fn bytes_stream(
        source: impl Send + AsyncRead,
    ) -> impl Send + Stream<Item = object_rainbow::Result<(u64, Vec<u8>)>> {
        try_stream(async move |co| {
            let source = pin!(source);
            let mut stream = AsyncStreamCDC::with_level(
                source,
                0x_00_01_00_00,
                0x_01_00_00_00,
                0x_ff_ff_ff_ff,
                Normalization::Level1,
            );
            stream
                .as_stream()
                .map_ok(|chunk| (chunk.offset, chunk.data))
                .try_for_each(|chunk| async {
                    co.yield_(chunk).await;
                    Ok(())
                })
                .await
                .map_err(std::io::Error::from)?;
            Ok(())
        })
    }

    pub async fn in_memory<F: Future<Output = object_rainbow::Result<Chunk>>>(
        source: impl Send + AsyncRead,
        mut schedule: impl FnMut(Box<dyn Send + FnOnce() -> object_rainbow::Result<Chunk>>) -> F,
    ) -> object_rainbow::Result<Self> {
        let chunks = Self::bytes_stream(source)
            .map_ok(|(_, chunk)| schedule(Box::new(move || Chunk::new(&chunk))))
            .try_collect::<Vec<_>>()
            .await?;
        let chunks = futures_util::future::try_join_all(chunks).await?;
        Ok(Self { chunks })
    }

    pub fn as_stream(&self) -> impl '_ + Send + Stream<Item = object_rainbow::Result<Vec<u8>>> {
        futures_util::stream::iter(&self.chunks).then(|chunk| chunk.data())
    }

    pub fn as_async_read(&self) -> impl '_ + Send + AsyncRead {
        self.as_stream().map_err(|e| e.into()).into_async_read()
    }

    pub fn into_stream(self) -> impl Send + Stream<Item = object_rainbow::Result<Vec<u8>>> {
        futures_util::stream::iter(self.chunks).then(|chunk| chunk.into_data())
    }

    pub fn into_async_read(self) -> impl Send + AsyncRead {
        self.into_stream().map_err(|e| e.into()).into_async_read()
    }

    pub fn len(&self) -> object_rainbow::Result<usize> {
        self.chunks.iter().map(|chunk| chunk.len()).sum()
    }

    pub fn is_empty(&self) -> object_rainbow::Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn chunk_len(&self) -> usize {
        self.chunks.len()
    }
}

#[derive(ToOutput, InlineOutput)]
pub struct Chunk {
    len_lower: u16,
    data: Point<Vec<u8>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new(&[]).unwrap()
    }
}

impl Chunk {
    pub async fn data(&self) -> object_rainbow::Result<Vec<u8>> {
        let len = self.len()?;
        let mut data = self.data.fetch().await?;
        data.truncate(len);
        Ok(data)
    }

    pub async fn into_data(mut self) -> object_rainbow::Result<Vec<u8>> {
        let len = self.len()?;
        let mut data = self.data.fetch_take().await?;
        data.truncate(len);
        Ok(data)
    }

    pub fn new(data: &[u8]) -> object_rainbow::Result<Self> {
        let (tail, hash) = generate_tail(data)?;
        let len_lower = (data.len() % 65536) as u16;
        let data = [data, tail.as_slice()].concat().point();
        assert_eq!(hash, data.hash());
        Ok(Self { len_lower, data })
    }

    pub fn len(&self) -> object_rainbow::Result<usize> {
        let len = u64::from(self.len_lower)
            + (u64::from(derive_length_from_hash(self.data.hash())) << 16);
        let len = len
            .try_into()
            .map_err(|_| object_rainbow::Error::UnsupportedLength)?;
        Ok(len)
    }

    pub fn is_empty(&self) -> object_rainbow::Result<bool> {
        Ok(self.len()? == 0)
    }
}

pub fn generate_tail(data: &[u8]) -> object_rainbow::Result<(Vec<u8>, Hash)> {
    let diff = DiffHashes::default().data_hash();
    let mut hasher = Sha256::new();
    hasher.update(diff);
    hasher.update(data);
    let hasher = hasher;
    let target = data.len() >> 16;
    let target: u32 = target
        .try_into()
        .map_err(|_| object_rainbow::Error::UnsupportedLength)?;
    for len in 0..=16 {
        for tail in 0u128..(1 << (len * 8)) {
            let tail = tail.to_be_bytes()[(16 - len)..].to_vec();
            let mut hasher = hasher.clone();
            hasher.update(&tail);
            let hash = Hash::from_hasher(hasher);
            if derive_length_from_hash(hash) == target {
                return Ok((tail, hash));
            }
        }
    }
    Err(object_rainbow::error_operation!(
        "couldn't find tail in 16 bytes or less"
    ))
}

pub fn derive_length_from_hash(hash: Hash) -> u32 {
    derive_length(hash.reinterpret::<(u64, u64, u64, u64)>().0)
}

fn derive_length(source: u64) -> u32 {
    const SIZE: u32 = u64::BITS;
    const HEAD_SIZE: u32 = 4;
    const TAIL_SIZE: u32 = SIZE - HEAD_SIZE;
    const TAIL_MASK: u64 = (1 << TAIL_SIZE) - 1;
    const BASE_BITS: u32 = 1;
    const MAX_BITS: u32 = BASE_BITS + ((1 << HEAD_SIZE) - 1) - 1;
    const MAX_GARBAGE: u32 = TAIL_SIZE - BASE_BITS;
    const_assert_eq!(MAX_BITS, 15);
    let head = (source >> TAIL_SIZE) as u32;
    let tail = source & TAIL_MASK;
    let garbage = MAX_GARBAGE - head.saturating_sub(1);
    let extra = (tail >> garbage) as u32;
    let main = if head == 0 {
        0
    } else {
        1 << (BASE_BITS - 1 + head)
    };
    main | extra
}

#[test]
#[expect(clippy::unusual_byte_groupings)]
fn cases() {
    let f = derive_length;
    assert_eq!(
        f(0b_0000_0_11111111111111111111111111111111111111111111111111111111111),
        0b_0,
    );
    assert_eq!(
        f(0b_0000_1_00000000000000000000000000000000000000000000000000000000000),
        0b_1,
    );
    assert_eq!(
        f(0b_0001_0_11111111111111111111111111111111111111111111111111111111111),
        0b_10,
    );
    assert_eq!(
        f(0b_0001_1_00000000000000000000000000000000000000000000000000000000000),
        0b_11,
    );
    assert_eq!(
        f(0b_0010_00_1111111111111111111111111111111111111111111111111111111111),
        0b_100,
    );
    assert_eq!(
        f(0b_0010_11_0000000000000000000000000000000000000000000000000000000000),
        0b_111,
    );
    assert_eq!(
        f(0b_0100_0000_11111111111111111111111111111111111111111111111111111111),
        0b_10000,
    );
    assert_eq!(
        f(0b_0100_1111_00000000000000000000000000000000000000000000000000000000),
        0b_11111,
    );
    assert_eq!(
        f(0b_1000_00000000_1111111111111111111111111111111111111111111111111111),
        0b_100000000,
    );
    assert_eq!(
        f(0b_1000_11111111_0000000000000000000000000000000000000000000000000000),
        0b_111111111,
    );
    assert_eq!(
        f(0b_1111_000000000000000_111111111111111111111111111111111111111111111),
        0b_1000000000000000,
    );
    assert_eq!(
        f(0b_1111_111111111111111_000000000000000000000000000000000000000000000),
        0b_1111111111111111,
    );
}
