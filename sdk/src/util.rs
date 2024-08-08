use bytes::{Bytes, BytesMut};
use futures_util::{Stream, StreamExt};
use anyhow::Result;

pub async fn get_body_max_size(
  stream: impl Stream<Item = reqwest::Result<Bytes>> + 'static,
  max_size: usize,
) -> Result<Bytes> {
  let mut max = 0;
  let mut b = BytesMut::new();
  let mut stream = Box::pin(stream);
  println!("starting {} ", max_size);
  while let Some(chunk) = stream.as_mut().next().await {
      let chunk_res = chunk?;
      let chunk = BytesMut::from(chunk_res.as_ref());
      let l = chunk.len();
      max += l;
      println!("chunk size {}", l);  
      println!("max size {}", max);
      if max > max_size {
          return Err(anyhow::anyhow!("Max size exceeded"));
      }
      b.extend_from_slice(&chunk);
  }
  Ok(b.into())
}