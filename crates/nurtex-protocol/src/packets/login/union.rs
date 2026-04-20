use nurtex_derive::PacketUnion;

use crate::packets::login::{
  ClientsideCookieRequest, ClientsideEncryptionRequest, ClientsideLoginDisconnect, ClientsideLoginSuccess, ClientsidePluginRequest, ClientsideSetCompression,
  ServersideCookieResponse, ServersideEncryptionResponse, ServersideLoginAcknowledged, ServersideLoginStart, ServersidePluginResponse,
};

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideLoginPacket {
  #[packet_id = "0x00"]
  Disconnect(ClientsideLoginDisconnect),
  #[packet_id = "0x01"]
  EncryptionRequest(ClientsideEncryptionRequest),
  #[packet_id = "0x02"]
  LoginSuccess(ClientsideLoginSuccess),
  #[packet_id = "0x03"]
  Compression(ClientsideSetCompression),
  #[packet_id = "0x04"]
  PluginRequest(ClientsidePluginRequest),
  #[packet_id = "0x05"]
  CookieRequest(ClientsideCookieRequest),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideLoginPacket {
  #[packet_id = "0x00"]
  LoginStart(ServersideLoginStart),
  #[packet_id = "0x01"]
  EncryptionResponse(ServersideEncryptionResponse),
  #[packet_id = "0x02"]
  PluginResponse(ServersidePluginResponse),
  #[packet_id = "0x03"]
  LoginAcknowledged(ServersideLoginAcknowledged),
  #[packet_id = "0x04"]
  CookieResponse(ServersideCookieResponse),
}
