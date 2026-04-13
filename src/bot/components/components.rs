use std::io::{self};

use azalea_protocol::connect::Connection;
use azalea_protocol::packets::game::{ClientboundGamePacket, ServerboundGamePacket};

use crate::bot::components::experience::Experience;
use crate::bot::components::inventory::Inventory;
use crate::bot::components::position::Position;
use crate::bot::components::profile::Profile;
use crate::bot::components::rotation::Rotation;
use crate::bot::components::state::State;
use crate::bot::components::velocity::Velocity;
use crate::bot::physics::Physics;

/// Структура компонентов бота
#[derive(Debug)]
pub struct BotComponents {
  pub state: State,
  pub profile: Profile,
  pub position: Position,
  pub rotation: Rotation,
  pub velocity: Velocity,
  pub experience: Experience,
  pub inventory: Inventory,
}

impl Default for BotComponents {
  fn default() -> Self {
    Self {
      state: State::default(),
      profile: Profile::default(),
      position: Position::zero(),
      rotation: Rotation::zero(),
      velocity: Velocity::zero(),
      experience: Experience::default(),
      inventory: Inventory::default(),
    }
  }
}

impl BotComponents {
  /// Метод выполнения определённых операций с компонентами в физический тик
  pub async fn tick(&mut self, conn: &mut Connection<ClientboundGamePacket, ServerboundGamePacket>, physics: &mut Physics, storage: &crate::bot::world::Storage) -> io::Result<()> {
    physics.apply_physics(&mut self.velocity, &self.position, storage);

    self.position.apply_velocity(self.velocity);

    physics.send_movement_packets(conn, self.position, self.rotation, &mut self.velocity).await?;

    Ok(())
  }
}
