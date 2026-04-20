use nurtex_codec::Buffer;

#[derive(Clone, Debug, PartialEq)]
pub struct KnownPack {
  pub namespace: String,
  pub id: String,
  pub version: String,
}

impl Buffer for KnownPack {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      namespace: String::read_buf(buffer)?,
      id: String::read_buf(buffer)?,
      version: String::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.namespace.write_buf(buffer)?;
    self.id.write_buf(buffer)?;
    self.version.write_buf(buffer)?;
    Ok(())
  }
}