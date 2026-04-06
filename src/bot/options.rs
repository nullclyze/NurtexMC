use azalea_protocol::common::client_information::{ClientInformation, ParticleStatus};

use crate::bot::components::{Physics, Profile, State};

#[derive(Debug, PartialEq)]
pub enum BotStatus {
  Offline,
  Connecting,
  Online,
}

#[derive(Debug)]
pub struct BotComponents {
  pub physics: Physics,
  pub state: State,
  pub profile: Profile,
}

#[derive(Debug)]
pub struct BotInformation {
  /// Бренд бота, по умолчанию "vanilla"
  pub brand: String,

  /// Клиентская информация бота
  pub client: ClientInformation,
}

impl Default for BotInformation {
  fn default() -> Self {
    Self {
      brand: "vanilla".to_string(),
      client: ClientInformation {
        particle_status: ParticleStatus::Minimal,
        ..Default::default()
      },
    }
  }
}

#[derive(Debug)]
pub struct BotPlugins {
  pub auto_reconnect: AutoReconnectPlugin,
  pub auto_respawn: AutoRespawnPlugin,
  pub physics: PhysicsPlugin,
}

impl Default for BotPlugins {
  fn default() -> Self {
    Self {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true,
        reconnect_delay: 6000,
      },
      auto_respawn: AutoRespawnPlugin { enabled: true },
      physics: PhysicsPlugin { enabled: true },
    }
  }
}

#[derive(Debug)]
pub struct AutoReconnectPlugin {
  pub enabled: bool,
  pub reconnect_delay: u64,
}

#[derive(Debug)]
pub struct AutoRespawnPlugin {
  pub enabled: bool,
}

#[derive(Debug)]
pub struct PhysicsPlugin {
  pub enabled: bool,
}
