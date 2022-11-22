use std::convert::Infallible;

use actix_web::web::{Bytes, BytesMut};
use async_stream::AsyncStream;
use futures_util::Future;

pub fn bytes_to_stream(
    mut bytes_mut:  BytesMut,
) -> AsyncStream<Result<Bytes, Infallible>, impl Future<Output = ()>> {
    async_stream::stream! {
        while !bytes_mut.is_empty() {
            let out = if bytes_mut.len() > 8192 {
                bytes_mut.split_to(8192)
            } else {
                bytes_mut.split_to(bytes_mut.len())
            };
            yield Ok::<_, Infallible>(out.freeze());
        }
    }
}
