use nurtex_codec::Buffer;

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
  pub name: String,
  pub value: String,
  pub signature: Option<String>,
}

impl Buffer for Property {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      name: String::read_buf(buffer)?,
      value: String::read_buf(buffer)?,
      signature: Option::read_buf(buffer)?
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.name.write_buf(buffer)?;
    self.value.write_buf(buffer)?;
    self.signature.write_buf(buffer)?;
    Ok(())
  }
}