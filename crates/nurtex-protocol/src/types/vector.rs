use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};

/// Структура позиции
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Vector3 {
  /// Метод создания нового экземпляра `Vector3`
  pub fn new(x: f64, y: f64, z: f64) -> Self {
    Self { x, y, z }
  }

  /// Метод создания нулевого вектора
  pub fn zero() -> Self {
    Self { x: 0.0, y: 0.0, z: 0.0 }
  }

  /// Метод вычисления разницы между текущим и другим вектором
  pub fn delta(&self, other: Vector3) -> Self {
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    let dz = self.z - other.z;

    Self { x: dx, y: dy, z: dz }
  }
}

impl Buffer for Vector3 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      x: f64::read_buf(buffer)?,
      y: f64::read_buf(buffer)?,
      z: f64::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.x.write_buf(buffer)?;
    self.y.write_buf(buffer)?;
    self.z.write_buf(buffer)?;
    Ok(())
  }
}

/// Структура компактного представление вектора
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LpVector3 {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl LpVector3 {
  const MAX_QUANTIZED_VALUE: f64 = 32766.0;

  /// Метод создания нового экземпляра `LpVector3`
  pub fn new(x: f64, y: f64, z: f64) -> Self {
    Self { x, y, z }
  }

  /// Метод создания нулевого вектора
  pub fn zero() -> Self {
    Self { x: 0.0, y: 0.0, z: 0.0 }
  }

  /// Метод упаковки значения
  fn pack(value: f64) -> i64 {
    ((value * 0.5 + 0.5) * Self::MAX_QUANTIZED_VALUE).round() as i64
  }

  /// Метод распаковки значения
  fn unpack(value: i64) -> f64 {
    let v = (value & 32767) as f64;
    (v.min(Self::MAX_QUANTIZED_VALUE)) * 2.0 / Self::MAX_QUANTIZED_VALUE - 1.0
  }
}

impl Buffer for LpVector3 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let byte1 = u8::read_buf(buffer)? as i32;

    if byte1 == 0 {
      return Some(Self { x: 0.0, y: 0.0, z: 0.0 });
    }

    let byte2 = u8::read_buf(buffer)? as i32;
    let bytes3to4 = u32::read_buf(buffer)? as i64;

    let packed = (bytes3to4 << 16) | ((byte2 as i64) << 8) | (byte1 as i64);

    let mut scale_factor = (byte1 & 3) as i64;

    if (byte1 & 4) == 4 {
      scale_factor |= (i32::read_varint(buffer)? as i64 & 0xFFFFFFFF) << 2;
    }

    let scale_factor_d = scale_factor as f64;

    Some(Self {
      x: Self::unpack(packed >> 3) * scale_factor_d,
      y: Self::unpack(packed >> 18) * scale_factor_d,
      z: Self::unpack(packed >> 33) * scale_factor_d,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let max_coordinate = self.x.abs().max(self.y.abs().max(self.z.abs()));

    if max_coordinate < 3.051944088384301e-5 {
      0u8.write_buf(buffer)?;
    } else {
      let max_coordinate_i = max_coordinate as i64;
      let scale_factor = if max_coordinate > max_coordinate_i as f64 {
        max_coordinate_i + 1
      } else {
        max_coordinate_i
      };

      let need_continuation = (scale_factor & 3) != scale_factor;
      let packed_scale = if need_continuation { (scale_factor & 3) | 4 } else { scale_factor };

      let packed_x = Self::pack(self.x / scale_factor as f64) << 3;
      let packed_y = Self::pack(self.y / scale_factor as f64) << 18;
      let packed_z = Self::pack(self.z / scale_factor as f64) << 33;

      let packed = packed_z | packed_y | packed_x | packed_scale;

      ((packed & 0xFF) as u8).write_buf(buffer)?;
      (((packed >> 8) & 0xFF) as u8).write_buf(buffer)?;
      ((packed >> 16) as u32).write_buf(buffer)?;

      if need_continuation {
        ((scale_factor >> 2) as i32).write_varint(buffer)?;
      }
    }

    Ok(())
  }
}
