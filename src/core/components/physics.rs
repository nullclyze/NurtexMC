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
  pub fn apply_velocity(&mut self) {
    self.position += self.velocity;
  }

  pub fn apply_gravity(&mut self) {
    if !self.on_ground {
      self.velocity.y -= 0.08;
      self.velocity.y *= 0.98;
    } else {
      if self.velocity.y < 0.0 {
        self.velocity.y = 0.0;
      }
    }
  }

  pub fn apply_friction(&mut self) {
    if self.on_ground {
      self.velocity.x *= 0.6;
      self.velocity.z *= 0.6;
    } else {
      self.velocity.x *= 0.91;
      self.velocity.z *= 0.91;
    }
  }

  pub async fn update(
    &mut self,
    conn: &mut Connection<ClientboundGamePacket, ServerboundGamePacket>,
  ) -> io::Result<()> {
    self.apply_velocity();
    self.apply_gravity();
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
}
