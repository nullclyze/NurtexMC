use crate::{CONTINUE_BIT, SEGMENT_BITS, read_byte};
use std::io::{self, Cursor, Write};

/// Обёртка для типа `VarLong`
pub trait VarLong {
  /// Метод чтения `VarLong` из буффера
  fn read_varlong(buffer: &mut Cursor<&[u8]>) -> Option<i64>;
  
  /// Метод записи `VarLong` в буффер
  fn write_varlong(&self, buffer: &mut impl Write) -> io::Result<()>;
}

impl VarLong for i64 {
  fn read_varlong(buffer: &mut Cursor<&[u8]>) -> Option<i64> {
    let mut value = 0i64;
    let mut position = 0u32;

    loop {
      let byte = read_byte(buffer)?;
      value |= (((byte & SEGMENT_BITS) as u32) << position) as i64;

      if (byte & CONTINUE_BIT) == 0 {
        break;
      }

      position += 7;

      if position >= 64 {
        return None;
      }
    }

    Some(value)
  }

  fn write_varlong(&self, buffer: &mut impl Write) -> io::Result<()> {
    let mut array = [0];
    let mut value = *self;

    if value == 0 {
      buffer.write_all(&array)?;
      return Ok(());
    }

    while value != 0 {
      array[0] = (value & SEGMENT_BITS as i64) as u8;
      value = (value >> 7) & (i64::MAX >> 6);

      if value != 0 {
        array[0] |= CONTINUE_BIT;
      }

      buffer.write_all(&array)?;
    }

    Ok(())
  }
}
