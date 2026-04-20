use nurtex_codec::{Buffer, VarInt};

#[derive(Clone, Debug, PartialEq)]
pub struct ServerLink {
  pub label: ServerLinkLabel,
  pub url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerLinkLabel {
  BuiltIn(i32),
  Custom(String),
}

impl Buffer for ServerLink {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      label: {
        let is_built_in = bool::read_buf(buffer)?;

        if is_built_in {
          ServerLinkLabel::BuiltIn(i32::read_varint(buffer)?)
        } else {
          ServerLinkLabel::Custom(String::read_buf(buffer)?)
        }
      },
      url: String::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    match &self.label {
      ServerLinkLabel::BuiltIn(id) => {
        true.write_buf(buffer)?;
        id.write_varint(buffer)?;
      }
      ServerLinkLabel::Custom(text) => {
        false.write_buf(buffer)?;
        text.write_buf(buffer)?;
      }
    }

    self.url.write_buf(buffer)?;

    Ok(())
  }
}