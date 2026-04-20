use nurtex_derive::PacketUnion;

use crate::packets::status::{ClientsidePongResponse, ClientsideStatusResponse, ServersidePingRequest, ServersideStatusRequest};

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideStatusPacket {
  #[packet_id = "0x00"]
  StatusResponse(ClientsideStatusResponse),
  #[packet_id = "0x01"]
  PongResponse(ClientsidePongResponse),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideStatusPacket {
  #[packet_id = "0x00"]
  StatusRequest(ServersideStatusRequest),
  #[packet_id = "0x01"]
  PingRequest(ServersidePingRequest),
}
