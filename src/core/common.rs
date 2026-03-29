use azalea_protocol::packets::game::{ServerboundGamePacket, s_interact::InteractionHand};
use tokio::sync::mpsc;

use crate::core::components::{Physics, Profile, State};

#[derive(Clone, Debug)]
pub enum BotCommand {
  Chat(String),
  SetDirection {
    yaw: f32,
    pitch: f32,
  },
  SetPosition {
    x: f64,
    y: f64,
    z: f64,
  },
  SwingArm(InteractionHand),
  StartUseItem(InteractionHand),
  ReleaseUseItem,
  SendPacket(ServerboundGamePacket),
  Disconnect,
  Reconnect {
    server_host: String,
    server_port: u16,
    interval: u64,
  },
}

#[derive(Clone)]
pub struct BotTerminal {
  pub receiver: String,
  pub cmd: mpsc::Sender<BotCommand>,
}

impl BotTerminal {
  /// Метод отправки команды в терминал.
  pub async fn send(&self, command: BotCommand) {
    let _ = self.cmd.send(command).await;
  }

  /// Вспомогательный метод отправки команды Chat в терминал.
  pub async fn chat(&self, message: impl Into<String>) {
    self.send(BotCommand::Chat(message.into())).await;
  }

  /// Вспомогательный метод отправки команды Disconnect в терминал.
  pub async fn disconnect(&self) {
    self.send(BotCommand::Disconnect).await;
  }

  /// Вспомогательный метод отправки команды Disconnect в терминал.
  pub async fn reconnect(&self, server_host: impl Into<String>, server_port: u16, interval: u64) {
    self
      .send(BotCommand::Reconnect {
        server_host: server_host.into(),
        server_port,
        interval,
      })
      .await;
  }
}

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
