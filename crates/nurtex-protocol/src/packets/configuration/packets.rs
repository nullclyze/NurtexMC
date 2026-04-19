use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};
use uuid::Uuid;

use crate::types::{AccurateHand, DisplayedSkinParts};

#[derive(Clone, Debug, PartialEq)]
pub struct MultisideKeepAlive {
  pub id: i64,
}

impl MultisideKeepAlive {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i64::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePing {
  pub id: i32,
}

impl ClientsidePing {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i32::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideCookieRequest {
  pub key: String,
}

impl ClientsideCookieRequest {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { key: String::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.key.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePluginMessage {
  pub channel: String,
  pub data: Vec<u8>,
}

impl ClientsidePluginMessage {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let channel = String::read_buf(buffer)?;
    let mut data = Vec::new();
    while let Some(byte) = u8::read_buf(buffer) {
      data.push(byte);
    }
    Some(Self { channel, data })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.channel.write_buf(buffer)?;
    for byte in &self.data {
      byte.write_buf(buffer)?;
    }
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideDisconnect {
  pub reason: Vec<u8>,
}

impl ClientsideDisconnect {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let remaining = buffer.get_ref().len() - buffer.position() as usize;
    let mut reason = vec![0u8; remaining];

    for byte in &mut reason {
      *byte = u8::read_buf(buffer)?;
    }

    Some(Self { reason })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    for byte in &self.reason {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideFinishConfiguration;

impl ClientsideFinishConfiguration {
  pub fn read(_buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self)
  }

  pub fn write(&self, _buffer: &mut impl Write) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideResetChat;

impl ClientsideResetChat {
  pub fn read(_buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self)
  }

  pub fn write(&self, _buffer: &mut impl Write) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideRegistryData {
  pub registry_id: String,
  pub raw_data: Vec<u8>,
}

impl ClientsideRegistryData {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let registry_id = String::read_buf(buffer)?;

    let remaining = buffer.get_ref().len() - buffer.position() as usize;
    let mut raw_data = vec![0u8; remaining];

    for byte in &mut raw_data {
      *byte = u8::read_buf(buffer)?;
    }

    Some(Self { registry_id, raw_data })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.registry_id.write_buf(buffer)?;

    for byte in &self.raw_data {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideRemoveResourcePack {
  pub uuid: Option<Uuid>,
}

impl ClientsideRemoveResourcePack {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let has_uuid = bool::read_buf(buffer)?;

    let uuid = if has_uuid { Some(Uuid::read_buf(buffer)?) } else { None };

    Some(Self { uuid })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.uuid.is_some().write_buf(buffer)?;

    if let Some(uuid) = &self.uuid {
      uuid.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideAddResourcePack {
  pub uuid: Uuid,
  pub url: String,
  pub hash: String,
  pub forced: bool,
  pub prompt_message: Option<String>,
}

impl ClientsideAddResourcePack {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let uuid = Uuid::read_buf(buffer)?;
    let url = String::read_buf(buffer)?;
    let hash = String::read_buf(buffer)?;
    let forced = bool::read_buf(buffer)?;
    let has_prompt = bool::read_buf(buffer)?;
    let prompt_message = if has_prompt { Some(String::read_buf(buffer)?) } else { None };

    Some(Self {
      uuid,
      url,
      hash,
      forced,
      prompt_message,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.uuid.write_buf(buffer)?;
    self.url.write_buf(buffer)?;
    self.hash.write_buf(buffer)?;
    self.forced.write_buf(buffer)?;
    self.prompt_message.is_some().write_buf(buffer)?;

    if let Some(msg) = &self.prompt_message {
      msg.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideStoreCookie {
  pub key: String,
  pub payload: Vec<u8>,
}

impl ClientsideStoreCookie {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let key = String::read_buf(buffer)?;
    let len = i32::read_varint(buffer)? as usize;
    let mut payload = vec![0u8; len];

    for byte in &mut payload {
      *byte = u8::read_buf(buffer)?;
    }

    Some(Self { key, payload })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.key.write_buf(buffer)?;
    (self.payload.len() as i32).write_varint(buffer)?;

    for byte in &self.payload {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideTransfer {
  pub server_host: String,
  pub server_port: i32,
}

impl ClientsideTransfer {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      server_host: String::read_buf(buffer)?,
      server_port: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.server_host.write_buf(buffer)?;
    self.server_port.write_varint(buffer)?;

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideFeatureFlags {
  pub features: Vec<String>,
}

impl ClientsideFeatureFlags {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let count = i32::read_varint(buffer)? as usize;
    let mut features = Vec::with_capacity(count);

    for _ in 0..count {
      features.push(String::read_buf(buffer)?);
    }

    Some(Self { features })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.features.len() as i32).write_varint(buffer)?;

    for feature in &self.features {
      feature.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideUpdateTags {
  pub tags: Vec<TagGroup>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagGroup {
  pub tag_type: String,
  pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
  pub name: String,
  pub entries: Vec<i32>,
}

impl ClientsideUpdateTags {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let groups_count = i32::read_varint(buffer)? as usize;
    let mut tags = Vec::with_capacity(groups_count);

    for _ in 0..groups_count {
      let tag_type = String::read_buf(buffer)?;
      let tags_count = i32::read_varint(buffer)? as usize;
      let mut group_tags = Vec::with_capacity(tags_count);

      for _ in 0..tags_count {
        let name = String::read_buf(buffer)?;
        let entries_count = i32::read_varint(buffer)? as usize;
        let mut entries = Vec::with_capacity(entries_count);

        for _ in 0..entries_count {
          entries.push(i32::read_varint(buffer)?);
        }

        group_tags.push(Tag { name, entries });
      }

      tags.push(TagGroup { tag_type, tags: group_tags });
    }

    Some(Self { tags })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.tags.len() as i32).write_varint(buffer)?;

    for group in &self.tags {
      group.tag_type.write_buf(buffer)?;
      (group.tags.len() as i32).write_varint(buffer)?;

      for tag in &group.tags {
        tag.name.write_buf(buffer)?;
        (tag.entries.len() as i32).write_varint(buffer)?;

        for entry in &tag.entries {
          entry.write_varint(buffer)?;
        }
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideKnownPacks {
  pub known_packs: Vec<KnownPack>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KnownPack {
  pub namespace: String,
  pub id: String,
  pub version: String,
}

impl ClientsideKnownPacks {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let count = i32::read_varint(buffer)? as usize;
    let mut known_packs = Vec::with_capacity(count);

    for _ in 0..count {
      known_packs.push(KnownPack {
        namespace: String::read_buf(buffer)?,
        id: String::read_buf(buffer)?,
        version: String::read_buf(buffer)?,
      });
    }

    Some(Self { known_packs })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.known_packs.len() as i32).write_varint(buffer)?;

    for pack in &self.known_packs {
      pack.namespace.write_buf(buffer)?;
      pack.id.write_buf(buffer)?;
      pack.version.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideCustomReportDetails {
  pub details: Vec<ReportDetail>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReportDetail {
  pub title: String,
  pub description: String,
}

impl ClientsideCustomReportDetails {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let count = i32::read_varint(buffer)? as usize;
    let mut details = Vec::with_capacity(count);

    for _ in 0..count {
      details.push(ReportDetail {
        title: String::read_buf(buffer)?,
        description: String::read_buf(buffer)?,
      });
    }

    Some(Self { details })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.details.len() as i32).write_varint(buffer)?;

    for detail in &self.details {
      detail.title.write_buf(buffer)?;
      detail.description.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideServerLinks {
  pub links: Vec<ServerLink>,
}

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

impl ClientsideServerLinks {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let count = i32::read_varint(buffer)? as usize;
    let mut links = Vec::with_capacity(count);

    for _ in 0..count {
      let is_built_in = bool::read_buf(buffer)?;

      let label = if is_built_in {
        ServerLinkLabel::BuiltIn(i32::read_varint(buffer)?)
      } else {
        ServerLinkLabel::Custom(String::read_buf(buffer)?)
      };

      let url = String::read_buf(buffer)?;

      links.push(ServerLink { label, url });
    }

    Some(Self { links })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.links.len() as i32).write_varint(buffer)?;

    for link in &self.links {
      match &link.label {
        ServerLinkLabel::BuiltIn(id) => {
          true.write_buf(buffer)?;
          id.write_varint(buffer)?;
        }
        ServerLinkLabel::Custom(text) => {
          false.write_buf(buffer)?;
          text.write_buf(buffer)?;
        }
      }

      link.url.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideClientInformation {
  pub locale: String,
  pub view_distance: i8,
  pub chat_mode: i32,
  pub chat_colors: bool,
  pub displayed_skin_parts: DisplayedSkinParts,
  pub main_hand: AccurateHand,
  pub enable_text_filtering: bool,
  pub allow_server_listings: bool,
  pub particle_status: i32,
}

impl ServersideClientInformation {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      locale: String::read_buf(buffer)?,
      view_distance: i8::read_buf(buffer)?,
      chat_mode: i32::read_varint(buffer)?,
      chat_colors: bool::read_buf(buffer)?,
      displayed_skin_parts: DisplayedSkinParts::from_mask(u8::read_buf(buffer)?),
      main_hand: AccurateHand::read_buf(buffer)?,
      enable_text_filtering: bool::read_buf(buffer)?,
      allow_server_listings: bool::read_buf(buffer)?,
      particle_status: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.locale.write_buf(buffer)?;
    self.view_distance.write_buf(buffer)?;
    self.chat_mode.write_varint(buffer)?;
    self.chat_colors.write_buf(buffer)?;
    u8::write_buf(&self.displayed_skin_parts.to_mask(), buffer)?;
    self.main_hand.write_buf(buffer)?;
    self.enable_text_filtering.write_buf(buffer)?;
    self.allow_server_listings.write_buf(buffer)?;
    self.particle_status.write_varint(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideCookieResponse {
  pub key: String,
  pub payload: Option<Vec<u8>>,
}

impl ServersideCookieResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let key = String::read_buf(buffer)?;
    let has_payload = bool::read_buf(buffer)?;

    let payload = if has_payload {
      let len = i32::read_varint(buffer)? as usize;
      let mut bytes = vec![0u8; len];

      for byte in &mut bytes {
        *byte = u8::read_buf(buffer)?;
      }

      Some(bytes)
    } else {
      None
    };

    Some(Self { key, payload })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.key.write_buf(buffer)?;
    self.payload.is_some().write_buf(buffer)?;

    if let Some(payload) = &self.payload {
      (payload.len() as i32).write_varint(buffer)?;

      for byte in payload {
        byte.write_buf(buffer)?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersidePong {
  pub id: i32,
}

impl ServersidePong {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i32::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersidePluginMessage {
  pub channel: String,
  pub data: Vec<u8>,
}

impl ServersidePluginMessage {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let channel = String::read_buf(buffer)?;
    let mut data = Vec::new();

    while let Some(byte) = u8::read_buf(buffer) {
      data.push(byte);
    }

    Some(Self { channel, data })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.channel.write_buf(buffer)?;

    for byte in &self.data {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideAcknowledgeFinishConfiguration;

impl ServersideAcknowledgeFinishConfiguration {
  pub fn read(_buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self)
  }

  pub fn write(&self, _buffer: &mut impl Write) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideResourcePackResponse {
  pub uuid: Uuid,
  pub state: ResourcePackState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourcePackState {
  SuccessfullyLoaded = 0,
  Declined = 1,
  FailedDownload = 2,
  Accepted = 3,
  Downloaded = 4,
  InvalidUrl = 5,
  FailedToReload = 6,
  Discarded = 7,
}

impl From<i32> for ResourcePackState {
  fn from(value: i32) -> Self {
    match value {
      0 => ResourcePackState::SuccessfullyLoaded,
      1 => ResourcePackState::Declined,
      2 => ResourcePackState::FailedDownload,
      3 => ResourcePackState::Accepted,
      4 => ResourcePackState::Downloaded,
      5 => ResourcePackState::InvalidUrl,
      6 => ResourcePackState::FailedToReload,
      7 => ResourcePackState::Discarded,
      _ => ResourcePackState::Declined,
    }
  }
}

impl ServersideResourcePackResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let uuid = Uuid::read_buf(buffer)?;
    let state_id = i32::read_varint(buffer)?;

    Some(Self { uuid, state: state_id.into() })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.uuid.write_buf(buffer)?;
    (self.state as i32).write_varint(buffer)?;

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideKnownPacks {
  pub known_packs: Vec<KnownPack>,
}

impl ServersideKnownPacks {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let count = i32::read_varint(buffer)? as usize;
    let mut known_packs = Vec::with_capacity(count);

    for _ in 0..count {
      known_packs.push(KnownPack {
        namespace: String::read_buf(buffer)?,
        id: String::read_buf(buffer)?,
        version: String::read_buf(buffer)?,
      });
    }

    Some(Self { known_packs })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.known_packs.len() as i32).write_varint(buffer)?;

    for pack in &self.known_packs {
      pack.namespace.write_buf(buffer)?;
      pack.id.write_buf(buffer)?;
      pack.version.write_buf(buffer)?;
    }

    Ok(())
  }
}
