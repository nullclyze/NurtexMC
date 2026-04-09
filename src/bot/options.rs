use azalea_protocol::common::client_information::{ClientInformation, ParticleStatus};

/// Статус подключения бота
#[derive(Debug, Clone, PartialEq)]
pub enum BotStatus {
  Offline,
  Connecting,
  Online,
}

/// Информация бота
#[derive(Debug, Clone)]
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

/// Плагины бота и их опции
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AutoReconnectPlugin {
  pub enabled: bool,
  pub reconnect_delay: u64,
}

#[derive(Debug, Clone)]
pub struct AutoRespawnPlugin {
  pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PhysicsPlugin {
  pub enabled: bool,
}
