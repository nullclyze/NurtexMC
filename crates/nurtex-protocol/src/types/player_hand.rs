use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};

/// Точная рука игрока (левая / правая)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum AccurateHand {
  Left,
  Right,
}

impl Buffer for AccurateHand {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_varint(buffer)?;

    match id {
      0 => Some(Self::Left),
      1 => Some(Self::Right),
      _ => None,
    }
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let id = match self {
      Self::Left => 0,
      Self::Right => 1,
    };

    id.write_varint(buffer)?;

    Ok(())
  }
}

/// Относительная рука игрока
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RelativeHand {
  MainHand,
  OffHand,
}

impl Buffer for RelativeHand {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_varint(buffer)?;

    match id {
      0 => Some(Self::MainHand),
      1 => Some(Self::OffHand),
      _ => None,
    }
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let id = match self {
      Self::MainHand => 0,
      Self::OffHand => 1,
    };

    id.write_varint(buffer)?;

    Ok(())
  }
}
