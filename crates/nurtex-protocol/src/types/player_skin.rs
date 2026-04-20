use nurtex_codec::Buffer;

/// Отображаемые части скина
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DisplayedSkinParts {
  pub cape: bool,
  pub jacket: bool,
  pub left_sleeve: bool,
  pub right_sleeve: bool,
  pub left_pants_leg: bool,
  pub right_pants_leg: bool,
  pub hat: bool,
}

impl Default for DisplayedSkinParts {
  fn default() -> Self {
    Self {
      cape: true,
      jacket: true,
      left_sleeve: true,
      right_sleeve: true,
      left_pants_leg: true,
      right_pants_leg: true,
      hat: true,
    }
  }
}

impl DisplayedSkinParts {
  /// Метод получения битовой маски из `DisplayedSkinParts`
  pub fn to_mask(&self) -> u8 {
    let mut mask = 0u8;

    if self.cape {
      mask |= 0x01;
    }
    if self.jacket {
      mask |= 0x02;
    }
    if self.left_sleeve {
      mask |= 0x04;
    }
    if self.right_sleeve {
      mask |= 0x08;
    }
    if self.left_pants_leg {
      mask |= 0x10;
    }
    if self.right_pants_leg {
      mask |= 0x20;
    }
    if self.hat {
      mask |= 0x40;
    }

    mask
  }

  /// Метод получения `DisplayedSkinParts` из битовой маски
  pub fn from_mask(mask: u8) -> Self {
    Self {
      cape: (mask & 0x01) != 0,
      jacket: (mask & 0x02) != 0,
      left_sleeve: (mask & 0x04) != 0,
      right_sleeve: (mask & 0x08) != 0,
      left_pants_leg: (mask & 0x10) != 0,
      right_pants_leg: (mask & 0x20) != 0,
      hat: (mask & 0x40) != 0,
    }
  }
}

impl Buffer for DisplayedSkinParts {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self::from_mask(u8::read_buf(buffer)?))
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.to_mask().write_buf(buffer)
  }
}