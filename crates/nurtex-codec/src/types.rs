use std::io::{self, Cursor, Write};
use uuid::Uuid;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::{Buffer, VarInt, read_str, write_str};

use byteorder::{BE, ReadBytesExt, WriteBytesExt};

impl Buffer for i32 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i32::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_i32::<BE>(*self)
  }
}

impl Buffer for u32 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(i32::read_buf(buffer)? as u32)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    i32::write_buf(&(*self as i32), buffer)
  }
}

impl Buffer for u16 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    i16::read_buf(buffer).map(|i| i as u16)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    i16::write_buf(&(*self as i16), buffer)
  }
}

impl Buffer for i16 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i16::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_i16::<BE>(*self)
  }
}

impl Buffer for i64 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i64::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_i64::<BE>(*self)
  }
}

impl Buffer for u64 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    i64::read_buf(buffer).map(|i| i as u64)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_u64::<BE>(*self)
  }
}

impl Buffer for bool {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let byte = u8::read_buf(buffer)?;

    if byte > 1 {
      return None;
    }

    Some(byte != 0)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let byte = u8::from(*self);
    byte.write_buf(buffer)
  }
}

impl Buffer for u8 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_u8().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_u8(*self)
  }
}

impl Buffer for i8 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    u8::read_buf(buffer).map(|i| i as i8)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    (*self as u8).write_buf(buffer)
  }
}

impl Buffer for f32 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_f32::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_f32::<BE>(*self)
  }
}

impl Buffer for f64 {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    buffer.read_f64::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_f64::<BE>(*self)
  }
}

impl<K: Buffer + Eq + Hash, V: Buffer> Buffer for HashMap<K, V> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let length = i32::read_varint(buffer)? as usize;
    let mut contents = HashMap::with_capacity(usize::min(length, 65536));

    for _ in 0..length {
      contents.insert(K::read_buf(buffer)?, V::read_buf(buffer)?);
    }

    Some(contents)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.len() as i32).write_varint(buffer)?;

    for (key, value) in self {
      key.write_buf(buffer)?;
      value.write_buf(buffer)?;
    }

    Ok(())
  }
}

impl<T: Buffer> Buffer for Vec<T> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let length = i32::read_varint(buffer)? as usize;
    let mut contents = Vec::with_capacity(usize::min(length, 65536));

    for _ in 0..length {
      contents.push(T::read_buf(buffer)?);
    }

    Some(contents)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.len() as i32).write_varint(buffer)?;

    for item in self.iter() {
      T::write_buf(item, buffer)?;
    }

    Ok(())
  }
}

impl<T: Buffer> Buffer for Box<[T]> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let length = i32::read_varint(buffer)? as usize;
    let mut contents = Vec::with_capacity(usize::min(length, 65536));

    for _ in 0..length {
      contents.push(T::read_buf(buffer)?);
    }

    Some(contents.into_boxed_slice())
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.len() as i32).write_varint(buffer)?;

    for item in self.iter() {
      T::write_buf(item, buffer)?;
    }

    Ok(())
  }
}

impl Buffer for String {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    read_str(buffer).map(Into::into)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    write_str(buffer, self)
  }
}

impl Buffer for Box<str> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    read_str(buffer).map(Into::into)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    write_str(buffer, self)
  }
}

impl<T: Buffer> Buffer for Option<T> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    if bool::read_buf(buffer)? { Some(T::read_buf(buffer)) } else { None }
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    if let Some(s) = self {
      true.write_buf(buffer)?;
      s.write_buf(buffer)
    } else {
      false.write_buf(buffer)
    }
  }
}

impl<T: Buffer, const N: usize> Buffer for [T; N] {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let mut contents = Vec::with_capacity(N);

    for _ in 0..N {
      contents.push(T::read_buf(buffer)?);
    }

    contents.try_into().ok()
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    for item in self {
      item.write_buf(buffer)?;
    }

    Ok(())
  }
}

impl<T: Buffer> Buffer for Box<T> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    T::read_buf(buffer).map(Box::new)
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    T::write_buf(&**self, buffer)
  }
}

impl<A: Buffer, B: Buffer> Buffer for (A, B) {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some((A::read_buf(buffer)?, B::read_buf(buffer)?))
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.0.write_buf(buffer)?;
    self.1.write_buf(buffer)
  }
}

impl<T: Buffer> Buffer for Arc<T> {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Arc::new(T::read_buf(buffer)?))
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    T::write_buf(&**self, buffer)
  }
}

impl Buffer for Uuid {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let array = [u32::read_buf(buffer)?, u32::read_buf(buffer)?, u32::read_buf(buffer)?, u32::read_buf(buffer)?];

    let most = ((array[0] as u64) << 32) | ((array[1] as u64) & 0xffffffff);
    let least = ((array[2] as u64) << 32) | ((array[3] as u64) & 0xffffffff);

    Some(Uuid::from_u128(((most as u128) << 64) | least as u128))
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let most = (self.as_u128() >> 64) as u64;
    let least = (self.as_u128() & 0xffffffffffffffff) as u64;

    let [a, b, c, d] = [(most >> 32) as u32, most as u32, (least >> 32) as u32, least as u32];

    a.write_buf(buffer)?;
    b.write_buf(buffer)?;
    c.write_buf(buffer)?;
    d.write_buf(buffer)?;
    Ok(())
  }
}
