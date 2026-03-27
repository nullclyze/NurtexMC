use azalea_protocol::packets::game::ClientboundGamePacket;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum BotEvent<'e> {
  LoginFinished,
  ConfigurationFinished,
  Spawn,
  Death,
  Disconnect,
  Chat { sender_uuid: Option<Uuid>, message: String },
  Packet(&'e ClientboundGamePacket)
}
