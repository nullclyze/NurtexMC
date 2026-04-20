use flate2::read::ZlibDecoder;
use futures::StreamExt;
use nurtex_codec::VarInt;
use nurtex_encrypt::AesDecryptor;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use tokio::io::AsyncRead;
use tokio_util::bytes::Buf;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::ProtocolPacket;

fn parse_frame(buffer: &mut Cursor<Vec<u8>>) -> Option<Box<[u8]>> {
  let mut buffer_copy = Cursor::new(&buffer.get_ref()[buffer.position() as usize..]);

  let length = i32::read_varint(&mut buffer_copy)? as usize;

  if length > buffer_copy.remaining() {
    return None;
  }

  let varint_length = buffer.remaining() - buffer_copy.remaining();
  buffer.advance(varint_length);
  let data = buffer.get_ref()[buffer.position() as usize..buffer.position() as usize + length].to_vec();
  buffer.advance(length);

  if buffer.position() == buffer.get_ref().len() as u64 {
    buffer.get_mut().clear();
    buffer.get_mut().shrink_to(1024 * 64);
    buffer.set_position(0);
  }

  Some(data.into_boxed_slice())
}

pub fn deserialize_packet<P: ProtocolPacket + Debug>(stream: &mut Cursor<&[u8]>) -> Option<P> {
  let packet_id = i32::read_varint(stream)? as u32;
  P::read(packet_id, stream)
}

pub fn compression_decoder(stream: &mut Cursor<&[u8]>, compression_threshold: u32) -> Option<Box<[u8]>> {
  let n = i32::read_varint(stream)? as u32;

  if n == 0 {
    let buf = stream.get_ref()[stream.position() as usize..].to_vec().into_boxed_slice();
    stream.set_position(stream.get_ref().len() as u64);
    return Some(buf);
  }

  if n < compression_threshold {
    return None;
  }

  if n > 8388608 {
    return None;
  }

  let mut decoded_buf = Vec::with_capacity(n as usize);
  let mut decoder = ZlibDecoder::new(stream);
  decoder.read_to_end(&mut decoded_buf).ok()?;

  Some(decoded_buf.into_boxed_slice())
}

pub async fn read_packet<P: ProtocolPacket + Debug, R>(stream: &mut R, buffer: &mut Cursor<Vec<u8>>, compression_threshold: Option<u32>, cipher: &mut Option<AesDecryptor>) -> Option<P>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  let raw_packet = read_raw_packet(stream, buffer, compression_threshold, cipher).await?;
  let packet = deserialize_packet(&mut Cursor::new(&raw_packet))?;
  Some(packet)
}

pub async fn read_raw_packet<R>(stream: &mut R, buffer: &mut Cursor<Vec<u8>>, compression_threshold: Option<u32>, cipher: &mut Option<AesDecryptor>) -> Option<Box<[u8]>>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  loop {
    if let Some(buf) = read_raw_packet_from_buffer::<R>(buffer, compression_threshold) {
      return Some(buf);
    };

    let bytes = read_and_decrypt_frame(stream, cipher).await?;
    buffer.get_mut().extend_from_slice(&bytes);
  }
}

async fn read_and_decrypt_frame<R>(stream: &mut R, cipher: &mut Option<AesDecryptor>) -> Option<Box<[u8]>>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  let mut framed = FramedRead::new(stream, BytesCodec::new());

  let Some(message) = framed.next().await else {
    return None;
  };

  let bytes = message.ok()?;

  let mut bytes = bytes.to_vec().into_boxed_slice();

  if let Some(cipher) = cipher {
    nurtex_encrypt::decrypt_packet(cipher, &mut bytes);
  }

  Some(bytes)
}

pub fn read_raw_packet_from_buffer<R>(buffer: &mut Cursor<Vec<u8>>, compression_threshold: Option<u32>) -> Option<Box<[u8]>>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  let Some(mut buf) = parse_frame(buffer) else {
    return None;
  };

  if let Some(compression_threshold) = compression_threshold {
    buf = compression_decoder(&mut Cursor::new(&buf[..]), compression_threshold)?;
  }

  Some(buf)
}
