use nurtex_codec::Buffer;
use nurtex_derive::Packet;

use crate::types::ClientIntention;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideGreet {
  #[packet(varint)]
  pub protocol_version: i32,
  pub server_host: String,
  pub server_port: u16,
  pub intention: ClientIntention,
}
