use azalea_protocol::packets::game::ClientboundGamePacket;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChatPayload {
  pub sender_uuid: Option<Uuid>,
  pub message: String,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PacketPayload {
  pub packet: ClientboundGamePacket,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum BotEvent {
  LoginFinished,
  ConfigurationFinished,
  Spawn,
  Death,
  Disconnect,
  Chat(ChatPayload),
  Packet(PacketPayload),
}
