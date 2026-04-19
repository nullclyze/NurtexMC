use nurtex_codec::{Buffer, VarInt};
use std::io::{self, Cursor, Write};

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

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideGreet {
  pub protocol_version: i32,
  pub server_host: String,
  pub server_port: u16,
  pub intention: ClientIntention,
}

impl ServersideGreet {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      protocol_version: i32::read_varint(buffer)?,
      server_host: String::read_buf(buffer)?,
      server_port: u16::read_buf(buffer)?,
      intention: ClientIntention::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.protocol_version.write_varint(buffer)?;
    self.server_host.write_buf(buffer)?;
    self.server_port.write_buf(buffer)?;
    self.intention.write_buf(buffer)?;
    Ok(())
  }
}
