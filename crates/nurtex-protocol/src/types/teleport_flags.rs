use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Флаги телепортации
#[derive(Clone, Debug, PartialEq)]
pub struct TeleportFlags {
  pub relative_x: bool,
  pub relative_y: bool,
  pub relative_z: bool,
  pub relative_yaw: bool,
  pub relative_pitch: bool,
  pub relative_velocity_x: bool,
  pub relative_velocity_y: bool,
  pub relative_velocity_z: bool,
  pub rotate_velocity: bool,
}

impl Buffer for TeleportFlags {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let flags = i32::read_buf(buffer)?;
    Some(Self {
      relative_x: (flags & 0x0001) != 0,
      relative_y: (flags & 0x0002) != 0,
      relative_z: (flags & 0x0004) != 0,
      relative_yaw: (flags & 0x0008) != 0,
      relative_pitch: (flags & 0x0010) != 0,
      relative_velocity_x: (flags & 0x0020) != 0,
      relative_velocity_y: (flags & 0x0040) != 0,
      relative_velocity_z: (flags & 0x0080) != 0,
      rotate_velocity: (flags & 0x0100) != 0,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let mut flags = 0i32;
    if self.relative_x {
      flags |= 0x0001;
    }
    if self.relative_y {
      flags |= 0x0002;
    }
    if self.relative_z {
      flags |= 0x0004;
    }
    if self.relative_yaw {
      flags |= 0x0008;
    }
    if self.relative_pitch {
      flags |= 0x0010;
    }
    if self.relative_velocity_x {
      flags |= 0x0020;
    }
    if self.relative_velocity_y {
      flags |= 0x0040;
    }
    if self.relative_velocity_z {
      flags |= 0x0080;
    }
    if self.rotate_velocity {
      flags |= 0x0100;
    }
    flags.write_buf(buffer)
  }
}
