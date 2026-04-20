use std::io::{self, Cursor, Write};

/// Трейт пакета протокола
pub trait ProtocolPacket
where
  Self: Sized,
{
  /// Метод получения ID текущего пакета
  fn id(&self) -> u32;

  /// Метод чтения данных определённого пакета из буффера
  fn read(id: u32, buffer: &mut Cursor<&[u8]>) -> Option<Self>;

  /// Метод записи данных пакета в буффер
  fn write(&self, buffer: &mut impl Write) -> io::Result<()>;
}

/// Трейт для получения образца пакета
pub trait IntoPacket<T> {
  fn into_packet(self) -> T;
}
