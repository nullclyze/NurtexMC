use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};

/// Структура последнего видимого сообщения
#[derive(Debug, Clone, PartialEq)]
pub struct LastSeenMessage {
  pub index: i32,
  pub signature: Option<Vec<u8>>,
}

impl Buffer for LastSeenMessage {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let index = i32::read_varint(buffer)?;
    let signature: Option<Vec<u8>> = Option::read_buf(buffer)?;

    Some(Self { index, signature })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.index.write_varint(buffer)?;
    self.signature.write_buf(buffer)?;
    Ok(())
  }
}
