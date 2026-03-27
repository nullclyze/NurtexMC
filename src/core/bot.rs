#![allow(dead_code)]

use std::io::{self, Error, ErrorKind};
use std::net::ToSocketAddrs;
use std::pin::Pin;

use azalea_protocol::connect::Connection;
use azalea_protocol::packets::game::s_interact::InteractionHand;
use azalea_protocol::packets::game::{ClientboundGamePacket, ServerboundGamePacket};
use azalea_protocol::packets::handshake::s_intention::ServerboundIntention;
use azalea_protocol::packets::login::s_hello::ServerboundHello;
use azalea_protocol::packets::login::s_login_acknowledged::ServerboundLoginAcknowledged;
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::components::{Physics, State};
use crate::core::default::{default_command_processor, default_packet_processor};
use crate::core::events::BotEvent;
use crate::core::handler::{handle_configuration, handle_login};
use crate::utils::sleep;

pub type PacketProcessorFn = for<'a> fn(
  &'a mut Bot,
  ClientboundGamePacket,
) -> Pin<
  Box<dyn std::future::Future<Output = io::Result<bool>> + Send + 'a>,
>;
pub type CommandProcessorFn = for<'a> fn(
  &'a mut Bot,
  BotCommand,
) -> Pin<
  Box<dyn std::future::Future<Output = io::Result<bool>> + Send + 'a>,
>;
pub type EventListenerFn = fn(&mut Bot, BotEvent) -> io::Result<()>;

#[derive(Clone, Debug)]
pub enum BotCommand {
  Chat(String),
  SetDirection { yaw: f32, pitch: f32 },
  SetPosition { x: f64, y: f64, z: f64 },
  SwingArm(InteractionHand),
  StartUseItem(InteractionHand),
  ReleaseUseItem,
  SendPacket(ServerboundGamePacket),
  Disconnect,
}

#[derive(Clone)]
pub struct BotTerminal {
  pub receiver: String,
  pub cmd: mpsc::Sender<BotCommand>
}

impl BotTerminal {
  pub async fn send(&self, command: BotCommand) {
    let _ = self.cmd.send(command).await;
  }
}

pub struct Bot {
  pub username: String,
  pub uuid: Uuid,
  pub connection: Option<Connection<ClientboundGamePacket, ServerboundGamePacket>>,
  pub command_receiver: mpsc::Receiver<BotCommand>,
  pub entity_id: Option<i32>,
  pub components: BotComponents,
  pub plugins: BotPlugins,
  packet_processor: PacketProcessorFn,
  command_processor: CommandProcessorFn,
  event_listener: Option<EventListenerFn>,
}

#[derive(Debug)]
pub struct BotComponents {
  pub physics: Physics,
  pub state: State,
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

impl Bot {
  pub fn new(username: &str, uuid: Uuid) -> (Self, BotTerminal) {
    let (sender, receiver) = mpsc::channel(100);

    let bot = Self {
      username: username.to_string(),
      uuid,
      connection: None,
      command_receiver: receiver,
      entity_id: None,
      components: BotComponents {
        physics: Physics::default(),
        state: State::default(),
      },
      plugins: BotPlugins::default(),
      packet_processor: default_packet_processor,
      command_processor: default_command_processor,
      event_listener: None,
    };

    let terminal = BotTerminal {
      receiver: username.to_string(),
      cmd: sender
    };
    
    (bot, terminal)
  }

  /// Метод установки плагинов бота.
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }

  /// Метод установки обработчика пакетов.
  pub fn set_packet_processor(mut self, processor: PacketProcessorFn) -> Self {
    self.packet_processor = processor;
    self
  }

  /// Метод установки обработчика команд.
  pub fn set_command_processor(mut self, processor: CommandProcessorFn) -> Self {
    self.command_processor = processor;
    self
  }

  /// Метод установки слушателя событий.
  pub fn set_event_listener(mut self, listener: EventListenerFn) -> Self {
    self.event_listener = Some(listener);
    self
  }

  /// Метод отправки события всем слушателям.
  pub fn emit_event(&mut self, event: BotEvent) {
    if let Some(listener) = self.event_listener {
      let _ = listener(self, event);
    }
  }

  /// Метод создания соединения с сервером и запуска `event_loop`.
  async fn start(&mut self, server_host: &str, server_port: u16) -> io::Result<()> {
    self.connection = None;

    let address_string = format!("{}:{}", server_host, server_port);

    let Some(address) = (match address_string.to_socket_addrs() {
      Ok(mut i) => i.next(),
      Err(err) => {
        return Err(err);
      }
    }) else {
      return Err(io::Error::new(
        io::ErrorKind::AddrNotAvailable,
        "Failed to retrieve socket address",
      ));
    };

    let mut conn = match Connection::new(&address).await {
      Ok(c) => c,
      Err(err) => {
        return Err(Error::new(
          ErrorKind::ConnectionRefused,
          format!(
            "Bot {} could not connect to {}: {}",
            self.username,
            &address,
            err.to_string()
          ),
        ));
      }
    };

    conn
      .write(ServerboundIntention {
        protocol_version: PROTOCOL_VERSION,
        hostname: server_host.to_string(),
        port: server_port,
        intention: ClientIntention::Login,
      })
      .await?;

    let mut conn = conn.login();
    conn
      .write(ServerboundHello {
        name: self.username.clone(),
        profile_id: self.uuid,
      })
      .await?;

    handle_login(&mut conn).await?;
    conn.write(ServerboundLoginAcknowledged {}).await?;

    self.emit_event(BotEvent::LoginFinished);

    let mut conn = conn.config();
    handle_configuration(&mut conn).await?;

    self.emit_event(BotEvent::ConfigurationFinished);

    let conn = conn.game();
    self.connection = Some(conn);

    self.event_loop().await?;

    Ok(())
  }

  /// Метод, который подключает бота к серверу, ловит его ошибки и корректно обрабатывает их.
  pub async fn connect_to(&mut self, server_host: &str, server_port: u16) -> io::Result<()> {
    loop {
      match self.start(server_host, server_port).await {
        Ok(_) => {
          break;
        }
        Err(err) => match err.kind() {
          ErrorKind::ConnectionRefused
          | ErrorKind::ConnectionReset
          | ErrorKind::ConnectionAborted => {
            self.emit_event(BotEvent::Disconnect);

            if self.plugins.auto_reconnect.enabled {
              sleep(self.plugins.auto_reconnect.reconnect_delay).await;
            } else {
              return Err(err);
            }
          }
          _ => {
            return Err(err);
          }
        },
      }
    }

    Ok(())
  }

  /// Метод основного цикла событий.
  async fn event_loop(&mut self) -> io::Result<()> {
    let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
      let Some(conn) = &mut self.connection else {
        continue;
      };

      tokio::select! {
        Ok(packet) = conn.read() => {
          match (self.packet_processor)(self, packet).await {
            Ok(true) => continue,
            Ok(false) => return Ok(()),
            Err(e) => return Err(e),
          }
        }

        Some(command) = self.command_receiver.recv() => {
          match (self.command_processor)(self, command).await {
            Ok(true) => continue,
            Ok(false) => return Ok(()),
            Err(e) => return Err(e),
          }
        }

        _ = tick_interval.tick() => {
          if let Err(e) = self.tick().await {
            return Err(e);
          }
        }
      }
    }
  }

  /// Метод проверки некого Entity ID на сходство с Entity ID текущего бота.
  pub fn is_this_my_entity_id(&self, id: i32) -> bool {
    if let Some(entity_id) = self.entity_id {
      if id == entity_id {
        return true;
      }
    }

    false
  }

  /// Метод выполнения определённых операций в каждый физический тик.
  async fn tick(&mut self) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Ok(());
    };

    if self.plugins.physics.enabled {
      self.components.physics.update(conn).await?;
    }

    Ok(())
  }

  /// Метод закрытия TcpStream (отключение от сервера).
  pub async fn disconnect(&mut self) -> io::Result<()> {
    let Some(conn) = self.connection.take() else {
      return Ok(());
    };

    let mut stream = match conn.unwrap() {
      Ok(s) => s,
      Err(err) => {
        return Err(Error::new(
          ErrorKind::Other,
          format!(
            "Bot {} could not disconnect: {}",
            self.username,
            err.to_string()
          ),
        ));
      }
    };

    stream.shutdown().await?;

    Ok(())
  }

  /// Метод переподключения бота к серверу.
  pub async fn reconnect(&mut self, server_host: &str, server_port: u16, interval: u64) -> io::Result<()> {
    self.disconnect().await?;
    sleep(interval).await;
    self.connect_to(server_host, server_port).await?;
    Ok(())
  }
}
