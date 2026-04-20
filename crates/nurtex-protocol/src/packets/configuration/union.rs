use nurtex_derive::PacketUnion;

use crate::packets::configuration::{
  ClientsideAddResourcePack, ClientsideCookieRequest, ClientsideCustomReportDetails, ClientsideDisconnect, ClientsideFeatureFlags, ClientsideFinishConfiguration,
  ClientsideKnownPacks, ClientsidePing, ClientsidePluginMessage, ClientsideRegistryData, ClientsideRemoveResourcePack, ClientsideResetChat, ClientsideServerLinks,
  ClientsideStoreCookie, ClientsideTransfer, ClientsideUpdateTags, MultisideKeepAlive, ServersideAcknowledgeFinishConfiguration, ServersideClientInformation,
  ServersideCookieResponse, ServersideKnownPacks, ServersidePluginMessage, ServersidePong, ServersideResourcePackResponse,
};

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideConfigurationPacket {
  #[packet_id = "0x00"]
  CookieRequest(ClientsideCookieRequest),
  #[packet_id = "0x01"]
  PluginMessage(ClientsidePluginMessage),
  #[packet_id = "0x02"]
  Disconnect(ClientsideDisconnect),
  #[packet_id = "0x03"]
  FinishConfiguration(ClientsideFinishConfiguration),
  #[packet_id = "0x04"]
  KeepAlive(MultisideKeepAlive),
  #[packet_id = "0x05"]
  Ping(ClientsidePing),
  #[packet_id = "0x06"]
  ResetChat(ClientsideResetChat),
  #[packet_id = "0x07"]
  RegistryData(ClientsideRegistryData),
  #[packet_id = "0x08"]
  RemoveResourcePack(ClientsideRemoveResourcePack),
  #[packet_id = "0x09"]
  AddResourcePack(ClientsideAddResourcePack),
  #[packet_id = "0x0A"]
  StoreCookie(ClientsideStoreCookie),
  #[packet_id = "0x0B"]
  Transfer(ClientsideTransfer),
  #[packet_id = "0x0C"]
  FeatureFlags(ClientsideFeatureFlags),
  #[packet_id = "0x0D"]
  UpdateTags(ClientsideUpdateTags),
  #[packet_id = "0x0E"]
  KnownPacks(ClientsideKnownPacks),
  #[packet_id = "0x0F"]
  CustomReportDetails(ClientsideCustomReportDetails),
  #[packet_id = "0x10"]
  ServerLinks(ClientsideServerLinks),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideConfigurationPacket {
  #[packet_id = "0x00"]
  ClientInformation(ServersideClientInformation),
  #[packet_id = "0x01"]
  CookieResponse(ServersideCookieResponse),
  #[packet_id = "0x02"]
  PluginMessage(ServersidePluginMessage),
  #[packet_id = "0x03"]
  AcknowledgeFinishConfiguration(ServersideAcknowledgeFinishConfiguration),
  #[packet_id = "0x04"]
  KeepAlive(MultisideKeepAlive),
  #[packet_id = "0x05"]
  Pong(ServersidePong),
  #[packet_id = "0x06"]
  ResourcePackResponse(ServersideResourcePackResponse),
  #[packet_id = "0x07"]
  KnownPacks(ServersideKnownPacks),
}
