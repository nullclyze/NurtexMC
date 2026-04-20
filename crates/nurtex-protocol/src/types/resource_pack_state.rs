use nurtex_codec::{Buffer, VarInt};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourcePackState {
  SuccessfullyLoaded,
  Declined,
  FailedDownload,
  Accepted,
  Downloaded,
  InvalidUrl,
  FailedToReload,
  Discarded,
}

impl Buffer for ResourcePackState {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_varint(buffer)?;

    Some(match id {
      0 => Self::SuccessfullyLoaded,
      1 => Self::Declined,
      2 => Self::FailedDownload,
      3 => Self::Accepted,
      4 => Self::Downloaded,
      5 => Self::InvalidUrl,
      6 => Self::FailedToReload,
      7 => Self::Discarded,
      _ => return None
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::SuccessfullyLoaded => 0.write_varint(buffer)?,
      Self::Declined => 1.write_varint(buffer)?,
      Self::FailedDownload => 2.write_varint(buffer)?,
      Self::Accepted => 3.write_varint(buffer)?,
      Self::Downloaded => 4.write_varint(buffer)?,
      Self::InvalidUrl => 5.write_varint(buffer)?,
      Self::FailedToReload => 6.write_varint(buffer)?,
      Self::Discarded => 7.write_varint(buffer)?,
    }

    Ok(())
  }
}