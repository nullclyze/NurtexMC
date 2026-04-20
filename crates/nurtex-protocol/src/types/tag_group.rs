use nurtex_codec::{Buffer, VarInt};

#[derive(Clone, Debug, PartialEq)]
pub struct TagGroup {
  pub tag_type: String,
  pub tags: Vec<Tag>,
}

impl Buffer for TagGroup {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let tag_type = String::read_buf(buffer)?;
    let tags_count = i32::read_varint(buffer)? as usize;
    let mut group_tags = Vec::with_capacity(tags_count);

    for _ in 0..tags_count {
      let tag = Tag::read_buf(buffer)?;
      group_tags.push(tag);
    }

    Some(Self { tag_type, tags: group_tags })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.tag_type.write_buf(buffer)?;
    (self.tags.len() as i32).write_varint(buffer)?;

    for tag in &self.tags {
      tag.name.write_buf(buffer)?;
      (tag.entries.len() as i32).write_varint(buffer)?;

      for entry in &tag.entries {
        entry.write_varint(buffer)?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
  pub name: String,
  pub entries: Vec<i32>,
}

impl Buffer for Tag {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      name: String::read_buf(buffer)?,
      entries: {
        let entries_count = i32::read_varint(buffer)? as usize;
        let mut entries = Vec::with_capacity(entries_count);

        for _ in 0..entries_count {
          entries.push(i32::read_varint(buffer)?);
        }

        entries
      }
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.name.write_buf(buffer)?;
    (self.entries.len() as i32).write_varint(buffer)?;

    for entry in &self.entries {
      entry.write_varint(buffer)?;
    }

    Ok(())
  }
}