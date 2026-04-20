use nurtex_derive::PacketUnion;

use std::io::{self, Cursor, Write};

use crate::packets::handshake::ServersideGreet;
use crate::Packet;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideHandshakePacket {
  #[packet_id = "0x00"]
  Greet(ServersideGreet),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientsideHandshakePacket {}

impl Packet for ClientsideHandshakePacket {
  fn id(&self) -> u32 {
    match *self {}
  }

  fn read(_id: u32, _buf: &mut Cursor<&[u8]>) -> Option<Self> {
    None
  }

  fn write(&self, _buf: &mut impl Write) -> io::Result<()> {
    match *self {}
  }
}
