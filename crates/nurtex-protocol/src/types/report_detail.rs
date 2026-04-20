use nurtex_codec::Buffer;

#[derive(Clone, Debug, PartialEq)]
pub struct ReportDetail {
  pub title: String,
  pub description: String,
}

impl Buffer for ReportDetail {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      title: String::read_buf(buffer)?,
      description: String::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.title.write_buf(buffer)?;
    self.description.write_buf(buffer)?;
    Ok(())
  }
}