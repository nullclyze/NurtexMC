use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};

/// Команда клиента
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ClientCommand {
  PerformRespawn,
  RequestStats,
}

impl Buffer for ClientCommand {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_varint(buffer)?;

    match id {
      0 => Some(Self::PerformRespawn),
      1 => Some(Self::RequestStats),
      _ => None,
    }
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let id = match self {
      Self::PerformRespawn => 0,
      Self::RequestStats => 1,
    };

    id.write_varint(buffer)?;

    Ok(())
  }
}
