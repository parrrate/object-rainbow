use std::sync::Arc;

use futures_util::{SinkExt, StreamExt, TryStreamExt, future::try_join};
use object_rainbow::Fetch;
use object_rainbow_bridge::{consume, provide};
use object_rainbow_point::{IntoPoint, Point, RawPointInner};

fn main() -> object_rainbow::Result<()> {
    smol::block_on(async move {
        let (send_consume, recv_consume) = flume::bounded(0);
        let (send_provide, recv_provide) = flume::bounded(0);
        let provide = provide(
            send_provide
                .into_sink()
                .sink_map_err(|_| object_rainbow::Error::Interrupted),
            recv_consume.into_stream().map(Ok),
            futures_util::stream::once(core::future::ready(Ok((
                Arc::new(b"123".to_vec().point().point().point().point()) as _,
                b"test".to_vec(),
            )))),
        );
        let mut consume = consume(
            send_consume
                .into_sink()
                .sink_map_err(|_| object_rainbow::Error::Interrupted),
            recv_provide.into_stream().map(Ok),
        );
        let consume = async move {
            let Some((point, reason)) = consume.try_next().await? else {
                return Err(object_rainbow::error_operation!("where???"));
            };
            assert_eq!(reason, b"test");
            let read = async move {
                let point = RawPointInner::from_singular(point)
                    .cast::<Point<Point<Point<Vec<u8>>>>, _>(())
                    .into_point();
                let data = point
                    .fetch()
                    .await?
                    .fetch()
                    .await?
                    .fetch()
                    .await?
                    .fetch()
                    .await?;
                assert_eq!(data, b"123");
                println!("done");
                Ok(())
            };
            let consume = consume.try_for_each(|_| core::future::ready(Ok(())));
            try_join(read, consume).await?;
            Ok(())
        };
        try_join(provide, consume).await?;
        Ok(())
    })
}
