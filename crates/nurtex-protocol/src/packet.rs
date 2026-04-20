use std::io::{self, Cursor, Write};

/// Трейт пакета
pub trait Packet
where
  Self: Sized,
{
  fn id(&self) -> u32;
  fn read(id: u32, buffer: &mut Cursor<&[u8]>) -> Option<Self>;
  fn write(&self, buffer: &mut impl Write) -> io::Result<()>;
}

/// Трейт для получения образца пакета
pub trait IntoPacket<T> {
  fn into_packet(self) -> T;
}
