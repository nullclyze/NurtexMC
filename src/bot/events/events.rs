use std::sync::Arc;

use azalea_protocol::packets::game::ClientboundGamePacket;
use uuid::Uuid;

use crate::bot::{
  components::{position::Position, rotation::Rotation},
  world::StorageLock,
};

#[derive(Debug, Clone)]
pub struct DisconnectPayload {
  pub reason: String,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ChatPayload {
  pub sender_uuid: Option<Uuid>,
  pub message: String,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PositionPayload {
  pub position: Position,
  pub old_position: Position,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RotationPayload {
  pub rotation: Rotation,
  pub old_rotation: Rotation,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PacketPayload {
  pub packet: Arc<ClientboundGamePacket>,
  pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ChunkPayload {
  pub x: i32,
  pub z: i32,
  pub storage: StorageLock,
}

#[derive(Debug, Clone)]
pub enum BotEvent {
  LoginFinished,
  ConfigurationFinished,
  Spawn,
  Death,
  Disconnect(DisconnectPayload),
  Chat(ChatPayload),
  UpdatePosition(PositionPayload),
  UpdateRotation(RotationPayload),
  Packet(PacketPayload),
  ChunkLoaded(ChunkPayload),
}
