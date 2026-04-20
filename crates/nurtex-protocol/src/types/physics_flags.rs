use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Флаги телепортации
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsFlags {
  pub on_ground: bool,
  pub pushing_against_wall: bool,
}

impl Buffer for PhysicsFlags {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      on_ground: bool::read_buf(buffer)?,
      pushing_against_wall: bool::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.on_ground.write_buf(buffer)?;
    self.pushing_against_wall.write_buf(buffer)?;
    Ok(())
  }
}
