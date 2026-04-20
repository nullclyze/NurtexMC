use nurtex_codec::VarInt;

use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClientIntention {
  Status,
  Login,
}

impl From<i32> for ClientIntention {
  fn from(value: i32) -> Self {
    match value {
      1 => ClientIntention::Status,
      2 => ClientIntention::Login,
      _ => ClientIntention::Status,
    }
  }
}

impl Buffer for ClientIntention {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_varint(buffer)?;
    Some(id.into())
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let id = match self {
      ClientIntention::Status => 1,
      ClientIntention::Login => 2,
    };

    id.write_varint(buffer)
  }
}