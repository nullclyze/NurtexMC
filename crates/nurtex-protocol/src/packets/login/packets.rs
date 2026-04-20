use nurtex_codec::Buffer;
use nurtex_derive::Packet;

use crate::types::Property;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideLoginDisconnect {
  pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideEncryptionRequest {
  pub server_id: String,
  pub public_key: Vec<u8>,
  pub verify_token: Vec<u8>,
  pub should_authenticate: bool,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideLoginSuccess {
  pub uuid: uuid::Uuid,
  pub username: String,
  pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSetCompression {
  #[packet(varint)]
  pub compression_threshold: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePluginRequest {
  #[packet(varint)]
  pub message_id: i32,
  pub channel: String,
  #[packet(vec_end)]
  pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideCookieRequest {
  pub key: String,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideLoginStart {
  pub username: String,
  pub uuid: uuid::Uuid,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideEncryptionResponse {
  pub shared_secret: Vec<u8>,
  pub verify_token: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePluginResponse {
  #[packet(varint)]
  pub message_id: i32,
  #[packet(option)]
  pub data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideLoginAcknowledged;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideCookieResponse {
  pub key: String,
  #[packet(option)]
  pub payload: Option<Vec<u8>>,
}
