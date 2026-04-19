use crate::{CONTINUE_BIT, SEGMENT_BITS, read_byte};
use std::io::{self, Cursor, Write};

/// Обёртка для типа `VarInt`
pub trait VarInt {
  /// Метод чтения `VarInt` из буффера
  fn read_varint(buffer: &mut Cursor<&[u8]>) -> Option<i32>;
  
  /// Метод записи `VarInt` в буффер
  fn write_varint(&self, buffer: &mut impl Write) -> io::Result<()>;
}

impl VarInt for i32 {
  fn read_varint(buffer: &mut Cursor<&[u8]>) -> Option<i32> {
    let mut value = 0i32;
    let mut position = 0u32;

    loop {
      let byte = read_byte(buffer)?;
      value |= (((byte & SEGMENT_BITS) as u32) << position) as i32;

      if (byte & CONTINUE_BIT) == 0 {
        break;
      }

      position += 7;

      if position >= 32 {
        return None;
      }
    }

    Some(value)
  }

  fn write_varint(&self, buffer: &mut impl Write) -> io::Result<()> {
    let mut array = [0];
    let mut value = *self;

    if value == 0 {
      buffer.write_all(&array)?;
      return Ok(());
    }

    while value != 0 {
      array[0] = (value & SEGMENT_BITS as i32) as u8;
      value = (value >> 7) & (i32::MAX >> 6);

      if value != 0 {
        array[0] |= CONTINUE_BIT;
      }

      buffer.write_all(&array)?;
    }

    Ok(())
  }
}
