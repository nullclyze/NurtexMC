use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Структура ротации
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rotation {
  pub yaw: f32,
  pub pitch: f32,
}

impl Rotation {
  /// Метод создания нового экземпляра `Rotation`
  pub fn new(yaw: f32, pitch: f32) -> Self {
    Self { yaw, pitch }
  }

  /// Метод создания нулевой ротации
  pub fn zero() -> Self {
    Self { yaw: 0.0, pitch: 0.0 }
  }

  /// Метод вычисления разницы между текущей и другой ротацией
  pub fn delta(&self, other: Rotation) -> Self {
    let dyaw = self.yaw - other.yaw;
    let dpitch = self.pitch - other.pitch;

    Self { yaw: dyaw, pitch: dpitch }
  }
}

impl Buffer for Rotation {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      yaw: f32::read_buf(buffer)?,
      pitch: f32::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.yaw.write_buf(buffer)?;
    self.pitch.write_buf(buffer)?;
    Ok(())
  }
}