use std::io;

use azalea_core::position::Vec3;
use azalea_entity::LookDirection;
use azalea_protocol::{
  common::movements::MoveFlags,
  connect::Connection,
  packets::game::{ClientboundGamePacket, ServerboundGamePacket, ServerboundMovePlayerPosRot},
};

#[derive(Debug)]
pub struct Physics {
  pub position: Vec3,
  pub velocity: Vec3,
  pub look_direction: LookDirection,
  pub on_ground: bool,
  pub last_sent_position: Vec3,
  pub last_sent_look_direction: LookDirection,
}

impl Default for Physics {
  fn default() -> Self {
    Self {
      position: Vec3::new(0.0, 0.0, 0.0),
      velocity: Vec3::new(0.0, 0.0, 0.0),
      look_direction: LookDirection::default(),
      on_ground: false,
      last_sent_position: Vec3::new(0.0, 0.0, 0.0),
      last_sent_look_direction: LookDirection::default(),
    }
  }
}

impl Physics {
  /// Метод обновления физики.
  pub async fn update(
    &mut self,
    conn: &mut Connection<ClientboundGamePacket, ServerboundGamePacket>,
  ) -> io::Result<()> {
    self.apply_gravity();
    self.apply_movement();
    self.apply_friction();

    let pos_delta = self.position - self.last_sent_position;
    let is_pos_changed = pos_delta.length_squared() > 2.0e-4_f64.powi(2);

    let yaw_changed =
      (self.look_direction.y_rot() - self.last_sent_look_direction.y_rot()).abs() > 0.01;
    let pitch_changed =
      (self.look_direction.x_rot() - self.last_sent_look_direction.x_rot()).abs() > 0.01;
    let is_look_changed = yaw_changed || pitch_changed;

    if is_pos_changed || is_look_changed {
      conn
        .write(ServerboundGamePacket::MovePlayerPosRot(
          ServerboundMovePlayerPosRot {
            pos: self.position,
            look_direction: self.look_direction,
            flags: MoveFlags {
              on_ground: self.on_ground,
              horizontal_collision: false,
            },
          },
        ))
        .await?;

      self.last_sent_position = self.position;
      self.last_sent_look_direction = self.look_direction;
    }

    Ok(())
  }

  /// Метод применения гравитации.
  fn apply_gravity(&mut self) {
    if !self.on_ground {
      self.velocity.y -= 0.08;
      self.velocity.y *= 0.98;
    } else {
      if self.velocity.y < 0.0 {
        self.velocity.y = 0.0;
      }
    }
  }

  /// Метод применения движения.
  fn apply_movement(&mut self) {
    self.position += self.velocity;
  }

  /// Метод применения трения.
  fn apply_friction(&mut self) {
    let inertia = if self.on_ground { 0.546 } else { 0.91 };

    self.velocity.x *= inertia;
    self.velocity.z *= inertia;
    self.velocity.y *= 0.98;
  }
}
