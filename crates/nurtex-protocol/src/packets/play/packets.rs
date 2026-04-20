use nurtex_codec::Buffer;
use nurtex_derive::Packet;
use uuid::Uuid;

use crate::types::{ClientCommand, LpVector3, PhysicsFlags, RelativeHand, Rotation, TeleportFlags, Vector3};

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct MultisideKeepAlive {
  pub id: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePing {
  pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePingResponse {
  pub timestamp: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideDamageEvent {
  #[packet(varint)]
  pub entity_id: i32,
  #[packet(varint)]
  pub source_type_id: i32,
  #[packet(varint)]
  pub source_cause_id: i32,
  #[packet(varint)]
  pub source_direct_id: i32,
  pub source_position: Vector3,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideUpdateEntityPos {
  #[packet(varint)]
  pub entity_id: i32,
  pub delta_x: i16,
  pub delta_y: i16,
  pub delta_z: i16,
  pub on_ground: bool,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideUpdateEntityRot {
  #[packet(varint)]
  pub entity_id: i32,
  pub yaw_angle: i8,
  pub pitch_angle: i8,
  pub on_ground: bool,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideUpdateEntityPosRot {
  #[packet(varint)]
  pub entity_id: i32,
  pub delta_x: i16,
  pub delta_y: i16,
  pub delta_z: i16,
  pub yaw_angle: i8,
  pub pitch_angle: i8,
  pub on_ground: bool,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePlayerPosition {
  #[packet(varlong)]
  pub teleport_id: i64,
  pub position: Vector3,
  pub velocity: Vector3,
  pub rotation: Rotation,
  pub teleport_flags: TeleportFlags,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePlayerRotation {
  pub yaw: f32,
  pub relative_yaw: bool,
  pub pitch: f32,
  pub relative_pitch: bool,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePlayerLookAt {
  #[packet(varint)]
  pub gaze: i32,
  pub target_pos: Vector3,
  pub is_entity: bool,
  #[packet(varint)]
  pub entity_id: Option<i32>,
  #[packet(varint)]
  pub entity_gaze: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePlayerCombatKill {
  #[packet(varint)]
  pub player_id: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSetHealth {
  pub health: f32,
  #[packet(varint)]
  pub food: i32,
  pub food_saturation: f32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSetExperience {
  pub experience_bar: f32,
  #[packet(varint)]
  pub level: i32,
  #[packet(varint)]
  pub total_experience: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSetPassengers {
  #[packet(varint)]
  pub entity_id: i32,
  #[packet(vec_varint)]
  pub passengers: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSetEntityVelocity {
  #[packet(varint)]
  pub entity_id: i32,
  pub velocity: LpVector3,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideSpawnEntity {
  #[packet(varint)]
  pub entity_id: i32,
  pub entity_uuid: Uuid,
  #[packet(varint)]
  pub entity_type: i32,
  pub position: Vector3,
  pub velocity: LpVector3,
  pub angle_pitch: i8,
  pub angle_yaw: i8,
  pub angle_head_yaw: i8,
  #[packet(varint)]
  pub data: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideRemoveEntities {
  #[packet(vec_varint)]
  pub entities: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideDisconnect;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePlayerChat {
  #[packet(varint)]
  pub global_index: i32,
  pub sender_uuid: Uuid,
  #[packet(varint)]
  pub index: i32,
  #[packet(option)]
  pub message_signature: Option<Vec<u8>>,
  pub message: String,
  pub timestamp: i64,
  pub salt: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePong {
  pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePingRequest {
  pub timestamp: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideAcceptTeleportation {
  #[packet(varlong)]
  pub teleport_id: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideSwingArm {
  pub hand: RelativeHand,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideUseItem {
  pub hand: RelativeHand,
  #[packet(varint)]
  pub sequence: i32,
  pub rotation: Rotation,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideMovePlayerPos {
  pub position: Vector3,
  pub flags: PhysicsFlags,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideMovePlayerRot {
  pub rotation: Rotation,
  pub flags: PhysicsFlags,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideMovePlayerPosRot {
  pub position: Vector3,
  pub rotation: Rotation,
  pub flags: PhysicsFlags,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideMovePlayerStatusOnly {
  pub flags: PhysicsFlags,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideClientCommand {
  pub command: ClientCommand,
}
