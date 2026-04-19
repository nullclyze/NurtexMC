use std::io::{self, Cursor, Read, Write};

use nurtex_codec::{Buffer, VarInt, VarLong};
use uuid::Uuid;

use crate::types::{ClientCommand, LpVector3, PhysicsFlags, RelativeHand, Rotation, TeleportFlags, Vector3};

#[derive(Clone, Debug, PartialEq)]
pub struct MultisideKeepAlive {
  pub id: i64,
}

impl MultisideKeepAlive {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i64::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePing {
  pub id: i32,
}

impl ClientsidePing {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i32::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePingResponse {
  pub timestamp: i64,
}

impl ClientsidePingResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      timestamp: i64::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.timestamp.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideLogin {
  pub entity_id: i32,
  pub is_hardcore: bool,
  pub dimension_names: Vec<String>,
  pub max_players: i32,
  pub view_distance: i32,
  pub simulation_distance: i32,
  pub reduced_debug_info: bool,
  pub enable_respawn_screen: bool,
  pub do_limited_crafting: bool,
  pub dimension_type: i32,
  pub dimension_name: String,
  pub hashed_seed: i64,
  pub game_mode: u8,
  pub previous_game_mode: i8,
  pub is_debug: bool,
  pub is_flat: bool,
  pub has_death_location: bool,
  pub death_dimension_name: Option<String>,
  pub death_location: Option<Vector3>,
  pub portal_cooldown: i32,
  pub sea_level: i32,
  pub enforces_secure_chat: bool,
}

impl ClientsideLogin {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_buf(buffer)?,
      is_hardcore: bool::read_buf(buffer)?,
      dimension_names: {
        let count = i32::read_varint(buffer)? as usize;
        let mut names = Vec::with_capacity(count);
        for _ in 0..count {
          names.push(String::read_buf(buffer)?);
        }
        names
      },
      max_players: i32::read_varint(buffer)?,
      view_distance: i32::read_varint(buffer)?,
      simulation_distance: i32::read_varint(buffer)?,
      reduced_debug_info: bool::read_buf(buffer)?,
      enable_respawn_screen: bool::read_buf(buffer)?,
      do_limited_crafting: bool::read_buf(buffer)?,
      dimension_type: i32::read_varint(buffer)?,
      dimension_name: String::read_buf(buffer)?,
      hashed_seed: i64::read_buf(buffer)?,
      game_mode: u8::read_buf(buffer)?,
      previous_game_mode: i8::read_buf(buffer)?,
      is_debug: bool::read_buf(buffer)?,
      is_flat: bool::read_buf(buffer)?,
      has_death_location: bool::read_buf(buffer)?,
      death_dimension_name: { if bool::read_buf(buffer)? { Some(String::read_buf(buffer)?) } else { None } },
      death_location: { if bool::read_buf(buffer)? { Some(Vector3::read_buf(buffer)?) } else { None } },
      portal_cooldown: i32::read_varint(buffer)?,
      sea_level: i32::read_varint(buffer)?,
      enforces_secure_chat: bool::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_buf(buffer)?;
    self.is_hardcore.write_buf(buffer)?;

    (self.dimension_names.len() as i32).write_varint(buffer)?;
    for name in &self.dimension_names {
      name.write_buf(buffer)?;
    }

    self.max_players.write_varint(buffer)?;
    self.view_distance.write_varint(buffer)?;
    self.simulation_distance.write_varint(buffer)?;
    self.reduced_debug_info.write_buf(buffer)?;
    self.enable_respawn_screen.write_buf(buffer)?;
    self.do_limited_crafting.write_buf(buffer)?;
    self.dimension_type.write_varint(buffer)?;

    self.dimension_name.write_buf(buffer)?;

    self.hashed_seed.write_buf(buffer)?;
    self.game_mode.write_buf(buffer)?;
    self.previous_game_mode.write_buf(buffer)?;
    self.is_debug.write_buf(buffer)?;
    self.is_flat.write_buf(buffer)?;
    self.has_death_location.write_buf(buffer)?;

    if let Some(ref death_dim) = self.death_dimension_name {
      true.write_buf(buffer)?;
      death_dim.write_buf(buffer)?;
    } else {
      false.write_buf(buffer)?;
    }

    if let Some(ref death_pos) = self.death_location {
      true.write_buf(buffer)?;
      death_pos.write_buf(buffer)?;
    } else {
      false.write_buf(buffer)?;
    }

    self.portal_cooldown.write_varint(buffer)?;
    self.sea_level.write_varint(buffer)?;
    self.enforces_secure_chat.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideDamageEvent {
  pub entity_id: i32,
  pub source_type_id: i32,
  pub source_cause_id: i32,
  pub source_direct_id: i32,
  pub source_position: Vector3,
}

impl ClientsideDamageEvent {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      source_type_id: i32::read_varint(buffer)?,
      source_cause_id: i32::read_varint(buffer)?,
      source_direct_id: i32::read_varint(buffer)?,
      source_position: Vector3::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.source_type_id.write_varint(buffer)?;
    self.source_cause_id.write_varint(buffer)?;
    self.source_direct_id.write_varint(buffer)?;
    self.source_position.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideUpdateEntityPos {
  pub entity_id: i32,
  pub delta_x: i16,
  pub delta_y: i16,
  pub delta_z: i16,
  pub on_ground: bool,
}

impl ClientsideUpdateEntityPos {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      delta_x: i16::read_buf(buffer)?,
      delta_y: i16::read_buf(buffer)?,
      delta_z: i16::read_buf(buffer)?,
      on_ground: bool::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.delta_x.write_buf(buffer)?;
    self.delta_y.write_buf(buffer)?;
    self.delta_z.write_buf(buffer)?;
    self.on_ground.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideUpdateEntityRot {
  pub entity_id: i32,
  pub yaw_angle: i8,
  pub pitch_angle: i8,
  pub on_ground: bool,
}

impl ClientsideUpdateEntityRot {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      yaw_angle: i8::read_buf(buffer)?,
      pitch_angle: i8::read_buf(buffer)?,
      on_ground: bool::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.yaw_angle.write_buf(buffer)?;
    self.pitch_angle.write_buf(buffer)?;
    self.on_ground.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideUpdateEntityPosRot {
  pub entity_id: i32,
  pub delta_x: i16,
  pub delta_y: i16,
  pub delta_z: i16,
  pub yaw_angle: i8,
  pub pitch_angle: i8,
  pub on_ground: bool,
}

impl ClientsideUpdateEntityPosRot {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      delta_x: i16::read_buf(buffer)?,
      delta_y: i16::read_buf(buffer)?,
      delta_z: i16::read_buf(buffer)?,
      yaw_angle: i8::read_buf(buffer)?,
      pitch_angle: i8::read_buf(buffer)?,
      on_ground: bool::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.delta_x.write_buf(buffer)?;
    self.delta_y.write_buf(buffer)?;
    self.delta_z.write_buf(buffer)?;
    self.yaw_angle.write_buf(buffer)?;
    self.pitch_angle.write_buf(buffer)?;
    self.on_ground.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePlayerPosition {
  pub teleport_id: i64,
  pub position: Vector3,
  pub velocity: Vector3,
  pub rotation: Rotation,
  pub teleport_flags: TeleportFlags,
}

impl ClientsidePlayerPosition {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      teleport_id: i64::read_varlong(buffer)?,
      position: Vector3::read_buf(buffer)?,
      velocity: Vector3::read_buf(buffer)?,
      rotation: Rotation::read_buf(buffer)?,
      teleport_flags: TeleportFlags::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.teleport_id.write_varlong(buffer)?;
    self.position.write_buf(buffer)?;
    self.velocity.write_buf(buffer)?;
    self.rotation.write_buf(buffer)?;
    self.teleport_flags.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePlayerRotation {
  pub yaw: f32,
  pub relative_yaw: bool,
  pub pitch: f32,
  pub relative_pitch: bool,
}

impl ClientsidePlayerRotation {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      yaw: f32::read_buf(buffer)?,
      relative_yaw: bool::read_buf(buffer)?,
      pitch: f32::read_buf(buffer)?,
      relative_pitch: bool::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.yaw.write_buf(buffer)?;
    self.relative_yaw.write_buf(buffer)?;
    self.pitch.write_buf(buffer)?;
    self.relative_pitch.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePlayerLookAt {
  pub gaze: i32,
  pub target_pos: Vector3,
  pub is_entity: bool,
  pub entity_id: Option<i32>,
  pub entity_gaze: Option<i32>,
}

impl ClientsidePlayerLookAt {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      gaze: i32::read_varint(buffer)?,
      target_pos: Vector3::read_buf(buffer)?,
      is_entity: bool::read_buf(buffer)?,
      entity_id: { if bool::read_buf(buffer)? { Some(i32::read_varint(buffer)?) } else { None } },
      entity_gaze: { if bool::read_buf(buffer)? { Some(i32::read_varint(buffer)?) } else { None } },
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.gaze.write_varint(buffer)?;
    self.target_pos.write_buf(buffer)?;
    self.is_entity.write_buf(buffer)?;

    if let Some(entity_id) = self.entity_id {
      true.write_buf(buffer)?;
      entity_id.write_varint(buffer)?;
    } else {
      false.write_buf(buffer)?;
    }

    if let Some(entity_gaze) = self.entity_gaze {
      true.write_buf(buffer)?;
      entity_gaze.write_varint(buffer)?;
    } else {
      false.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePlayerCombatKill {
  pub player_id: i32,
}

impl ClientsidePlayerCombatKill {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      player_id: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.player_id.write_varint(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSetHealth {
  pub health: f32,
  pub food: i32,
  pub food_saturation: f32,
}

impl ClientsideSetHealth {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      health: f32::read_buf(buffer)?,
      food: i32::read_varint(buffer)?,
      food_saturation: f32::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.health.write_buf(buffer)?;
    self.food.write_varint(buffer)?;
    self.food_saturation.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSetExperience {
  pub experience_bar: f32,
  pub level: i32,
  pub total_experience: i32,
}

impl ClientsideSetExperience {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      experience_bar: f32::read_buf(buffer)?,
      level: i32::read_varint(buffer)?,
      total_experience: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.experience_bar.write_buf(buffer)?;
    self.level.write_varint(buffer)?;
    self.total_experience.write_varint(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSetPassengers {
  pub entity_id: i32,
  pub passengers: Vec<i32>,
}

impl ClientsideSetPassengers {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      passengers: {
        let count = i32::read_varint(buffer)? as usize;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
          vec.push(i32::read_varint(buffer)?);
        }
        vec
      },
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;

    (self.passengers.len() as i32).write_varint(buffer)?;
    for passenger in &self.passengers {
      passenger.write_varint(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSetEntityVelocity {
  pub entity_id: i32,
  pub velocity: LpVector3,
}

impl ClientsideSetEntityVelocity {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      velocity: LpVector3::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.velocity.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSpawnEntity {
  pub entity_id: i32,
  pub entity_uuid: Uuid,
  pub entity_type: i32,
  pub position: Vector3,
  pub velocity: LpVector3,
  pub angle_pitch: i8,
  pub angle_yaw: i8,
  pub angle_head_yaw: i8,
  pub data: i32,
}

impl ClientsideSpawnEntity {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entity_id: i32::read_varint(buffer)?,
      entity_uuid: Uuid::read_buf(buffer)?,
      entity_type: i32::read_varint(buffer)?,
      position: Vector3::read_buf(buffer)?,
      velocity: LpVector3::read_buf(buffer)?,
      angle_pitch: i8::read_buf(buffer)?,
      angle_yaw: i8::read_buf(buffer)?,
      angle_head_yaw: i8::read_buf(buffer)?,
      data: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.entity_id.write_varint(buffer)?;
    self.entity_uuid.write_buf(buffer)?;
    self.entity_type.write_varint(buffer)?;
    self.position.write_buf(buffer)?;
    self.velocity.write_buf(buffer)?;
    self.angle_pitch.write_buf(buffer)?;
    self.angle_yaw.write_buf(buffer)?;
    self.angle_head_yaw.write_buf(buffer)?;
    self.data.write_varint(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideRemoveEntities {
  pub entities: Vec<i32>,
}

impl ClientsideRemoveEntities {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      entities: {
        let count = i32::read_varint(buffer)? as usize;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
          vec.push(i32::read_varint(buffer)?);
        }
        vec
      },
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.entities.len() as i32).write_varint(buffer)?;
    for entity in &self.entities {
      entity.write_varint(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideDisconnect;

impl ClientsideDisconnect {
  pub fn read(_buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self)
  }

  pub fn write(&self, _buffer: &mut impl Write) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePlayerChat {
  pub global_index: i32,
  pub sender_uuid: Uuid,
  pub index: i32,
  pub message_signature: Option<Vec<u8>>,
  pub message: String,
  pub timestamp: i64,
  pub salt: i64,
  pub previous_messages: Vec<(i32, Option<Vec<u8>>)>,
}

impl ClientsidePlayerChat {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let global_index = i32::read_varint(buffer)?;
    let sender_uuid = Uuid::read_buf(buffer)?;
    let index = i32::read_varint(buffer)?;

    let message_signature = if bool::read_buf(buffer)? {
      let mut sig = vec![0u8; 256];
      buffer.read_exact(&mut sig).ok()?;
      Some(sig)
    } else {
      None
    };

    let message = String::read_buf(buffer)?;
    let timestamp = i64::read_buf(buffer)?;
    let salt = i64::read_buf(buffer)?;

    let previous_message_count = i32::read_varint(buffer)? as usize;
    let mut previous_messages = Vec::with_capacity(previous_message_count);
    for _ in 0..previous_message_count {
      let msg_id = i32::read_varint(buffer)?;
      let sig = if bool::read_buf(buffer)? {
        let mut s = vec![0u8; 256];
        buffer.read_exact(&mut s).ok()?;
        Some(s)
      } else {
        None
      };
      previous_messages.push((msg_id, sig));
    }

    Some(Self {
      global_index,
      sender_uuid,
      index,
      message_signature,
      message,
      timestamp,
      salt,
      previous_messages,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.global_index.write_varint(buffer)?;
    self.sender_uuid.write_buf(buffer)?;
    self.index.write_varint(buffer)?;

    if let Some(sig) = &self.message_signature {
      true.write_buf(buffer)?;
      buffer.write_all(sig)?;
    } else {
      false.write_buf(buffer)?;
    }

    self.message.write_buf(buffer)?;
    self.timestamp.write_buf(buffer)?;
    self.salt.write_buf(buffer)?;

    (self.previous_messages.len() as i32).write_varint(buffer)?;
    for (msg_id, sig) in &self.previous_messages {
      msg_id.write_varint(buffer)?;
      if let Some(s) = sig {
        true.write_buf(buffer)?;
        buffer.write_all(s)?;
      } else {
        false.write_buf(buffer)?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersidePong {
  pub id: i32,
}

impl ServersidePong {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { id: i32::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.id.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersidePingRequest {
  pub timestamp: i64,
}

impl ServersidePingRequest {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      timestamp: i64::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.timestamp.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideAcceptTeleportation {
  pub teleport_id: i64,
}

impl ServersideAcceptTeleportation {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      teleport_id: i64::read_varlong(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.teleport_id.write_varlong(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideSwingArm {
  pub hand: RelativeHand,
}

impl ServersideSwingArm {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      hand: RelativeHand::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.hand.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideUseItem {
  pub hand: RelativeHand,
  pub sequence: i32,
  pub rotation: Rotation,
}

impl ServersideUseItem {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      hand: RelativeHand::read_buf(buffer)?,
      sequence: i32::read_varint(buffer)?,
      rotation: Rotation::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.hand.write_buf(buffer)?;
    self.sequence.write_varint(buffer)?;
    self.hand.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideMovePlayerPos {
  pub position: Vector3,
  pub flags: PhysicsFlags,
}

impl ServersideMovePlayerPos {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      position: Vector3::read_buf(buffer)?,
      flags: PhysicsFlags::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.position.write_buf(buffer)?;
    self.flags.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideMovePlayerRot {
  pub rotation: Rotation,
  pub flags: PhysicsFlags,
}

impl ServersideMovePlayerRot {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      rotation: Rotation::read_buf(buffer)?,
      flags: PhysicsFlags::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.rotation.write_buf(buffer)?;
    self.flags.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideMovePlayerPosRot {
  pub position: Vector3,
  pub rotation: Rotation,
  pub flags: PhysicsFlags,
}

impl ServersideMovePlayerPosRot {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      position: Vector3::read_buf(buffer)?,
      rotation: Rotation::read_buf(buffer)?,
      flags: PhysicsFlags::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.position.write_buf(buffer)?;
    self.rotation.write_buf(buffer)?;
    self.flags.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideMovePlayerStatusOnly {
  pub flags: PhysicsFlags,
}

impl ServersideMovePlayerStatusOnly {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      flags: PhysicsFlags::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.flags.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideClientCommand {
  pub command: ClientCommand,
}

impl ServersideClientCommand {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      command: ClientCommand::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.command.write_buf(buffer)?;
    Ok(())
  }
}
