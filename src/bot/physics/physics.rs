use std::io;

use azalea_core::position::ChunkPos;
use azalea_protocol::{
  common::movements::MoveFlags,
  connect::Connection,
  packets::game::{ClientboundGamePacket, ServerboundGamePacket, ServerboundMovePlayerPos, ServerboundMovePlayerPosRot, ServerboundMovePlayerRot, ServerboundMovePlayerStatusOnly},
};

use crate::bot::components::{position::Position, rotation::Rotation, velocity::Velocity};
use crate::bot::world::Storage;

#[derive(Debug)]
pub struct Physics {
  pub on_ground: bool,
  pub last_on_ground: bool,
  pub last_sent_position: Position,
  pub last_sent_rotation: Rotation,
}

impl Default for Physics {
  fn default() -> Self {
    Self {
      on_ground: false,
      last_on_ground: false,
      last_sent_position: Position::zero(),
      last_sent_rotation: Rotation::zero(),
    }
  }
}

impl Physics {
  /// Метод применения физики
  pub fn apply_physics(&mut self, velocity: &mut Velocity, position: &Position, storage: &Storage) {
    let new_on_ground = storage.is_on_ground(&position.to_vec3());

    let pos_vec = position.to_vec3();
    let chunk_pos = ChunkPos::new((pos_vec.x.floor() as i32) >> 4, (pos_vec.z.floor() as i32) >> 4);

    if storage.is_chunk_loaded(&chunk_pos) {
      self.on_ground = new_on_ground;
    }

    self.apply_gravity(velocity);
    self.apply_friction(velocity);
  }

  /// Метод отправки пакета движения серверу
  pub async fn send_movement_packets(&mut self, conn: &mut Connection<ClientboundGamePacket, ServerboundGamePacket>, position: Position, rotation: Rotation) -> io::Result<()> {
    let pos_delta = position.delta(self.last_sent_position);
    let is_pos_changed = pos_delta != Position::zero();

    let rot_delta = rotation.delta(self.last_sent_rotation);
    let is_rot_changed = rot_delta != Rotation::zero();

    match (is_pos_changed, is_rot_changed) {
      (true, true) => {
        conn
          .write(ServerboundGamePacket::MovePlayerPosRot(ServerboundMovePlayerPosRot {
            pos: position.to_vec3(),
            look_direction: rotation.to_look_direction(),
            flags: MoveFlags {
              on_ground: self.on_ground,
              horizontal_collision: false,
            },
          }))
          .await?;
      }
      (true, false) => {
        conn
          .write(ServerboundGamePacket::MovePlayerPos(ServerboundMovePlayerPos {
            pos: position.to_vec3(),
            flags: MoveFlags {
              on_ground: self.on_ground,
              horizontal_collision: false,
            },
          }))
          .await?;
      }
      (false, true) => {
        conn
          .write(ServerboundGamePacket::MovePlayerRot(ServerboundMovePlayerRot {
            look_direction: rotation.to_look_direction(),
            flags: MoveFlags {
              on_ground: self.on_ground,
              horizontal_collision: false,
            },
          }))
          .await?;
      }
      (false, false) => {
        if self.last_on_ground != self.on_ground {
          conn
            .write(ServerboundGamePacket::MovePlayerStatusOnly(ServerboundMovePlayerStatusOnly {
              flags: MoveFlags {
                on_ground: self.on_ground,
                horizontal_collision: false,
              },
            }))
            .await?;

          self.last_on_ground = self.on_ground;
        }
      }
    }

    if is_pos_changed {
      self.last_sent_position = position;
    }

    if is_rot_changed {
      self.last_sent_rotation = rotation;
    }

    Ok(())
  }

  /// Метод применения гравитации
  fn apply_gravity(&mut self, velocity: &mut Velocity) {
    if !self.on_ground {
      velocity.y -= 0.08;
      velocity.y *= 0.98;
    } else {
      if velocity.y < 0.0 {
        velocity.y = 0.0;
      }
    }
  }

  /// Метод применения трения
  fn apply_friction(&mut self, velocity: &mut Velocity) {
    let inertia = if self.on_ground { 0.546 } else { 0.91 };

    velocity.x *= inertia;
    velocity.z *= inertia;
    velocity.y *= 0.98;
  }
}
