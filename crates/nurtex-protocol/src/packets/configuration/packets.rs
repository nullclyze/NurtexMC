use nurtex_codec::Buffer;
use nurtex_derive::Packet;

use crate::types::{AccurateHand, ChatMode, DisplayedSkinParts, KnownPack, ParticleStatus, ReportDetail, ResourcePackState, ServerLink, TagGroup, TextComponent};

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct MultisideKeepAlive {
  pub id: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePing {
  pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideCookieRequest {
  pub key: String,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePluginMessage {
  pub channel: String,
  #[packet(vec_end)]
  pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideDisconnect {
  pub reason: TextComponent,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideFinishConfiguration;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideResetChat;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideRegistryData {
  pub registry_id: String,
  #[packet(vec_end)]
  pub raw_data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideRemoveResourcePack {
  pub uuid: Option<uuid::Uuid>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideAddResourcePack {
  pub uuid: uuid::Uuid,
  pub url: String,
  pub hash: String,
  pub forced: bool,
  pub prompt_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideStoreCookie {
  pub key: String,
  #[packet(vec_end)]
  pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideTransfer {
  pub server_host: String,
  #[packet(varint)]
  pub server_port: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideFeatureFlags {
  pub features: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideUpdateTags {
  pub tags: Vec<TagGroup>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideKnownPacks {
  pub known_packs: Vec<KnownPack>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideCustomReportDetails {
  pub details: Vec<ReportDetail>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideServerLinks {
  pub links: Vec<ServerLink>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideClientInformation {
  pub locale: String,
  pub view_distance: i8,
  pub chat_mode: ChatMode,
  pub chat_colors: bool,
  pub displayed_skin_parts: DisplayedSkinParts,
  pub main_hand: AccurateHand,
  pub enable_text_filtering: bool,
  pub allow_server_listings: bool,
  pub particle_status: ParticleStatus,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideCookieResponse {
  pub key: String,
  pub payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePong {
  pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePluginMessage {
  pub channel: String,
  #[packet(vec_end)]
  pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideAcknowledgeFinishConfiguration;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideResourcePackResponse {
  pub uuid: uuid::Uuid,
  pub state: ResourcePackState,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideKnownPacks {
  pub known_packs: Vec<KnownPack>,
}
