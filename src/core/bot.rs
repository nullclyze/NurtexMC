#![allow(dead_code)]

use std::io::{self, Error, ErrorKind};
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use azalea_protocol::common::client_information::{ClientInformation, ParticleStatus};
use azalea_protocol::connect::Connection;
use azalea_protocol::packets::game::s_chat::LastSeenMessagesUpdate;
use azalea_protocol::packets::game::s_interact::InteractionHand;
use azalea_protocol::packets::game::{ClientboundGamePacket, ServerboundChat, ServerboundGamePacket};
use azalea_protocol::packets::handshake::s_intention::ServerboundIntention;
use azalea_protocol::packets::login::s_hello::ServerboundHello;
use azalea_protocol::packets::login::s_login_acknowledged::ServerboundLoginAcknowledged;
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::core::components::{Physics, Profile, State};
use crate::core::data::Storage;
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
  Reconnect { server_host: String, server_port: u16, interval: u64 }
}

#[derive(Clone)]
pub struct BotTerminal {
  pub receiver: String,
  pub cmd: mpsc::Sender<BotCommand>
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
    self.send(BotCommand::Reconnect { 
      server_host: server_host.into(), 
      server_port, 
      interval 
    }).await;
  }
}

pub struct Bot {
  pub status: BotStatus,
  pub username: String,
  pub uuid: Uuid,
  pub connection: Option<Connection<ClientboundGamePacket, ServerboundGamePacket>>,
  pub components: BotComponents,
  pub plugins: BotPlugins,
  pub storage: Storage,
  client_information: ClientInformation,
  command_receiver: mpsc::Receiver<BotCommand>,
  packet_processor: PacketProcessorFn,
  command_processor: CommandProcessorFn,
  event_listener: Option<EventListenerFn>,
}

#[derive(Debug, PartialEq)]
pub enum BotStatus {
  Offline,
  Connecting,
  Online
}

#[derive(Debug)]
pub struct BotComponents {
  pub physics: Physics,
  pub state: State,
  pub profile: Profile
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
  pub fn new(username: &str) -> (Self, BotTerminal) {
    let (sender, receiver) = mpsc::channel(100);

    let bot = Self {
      status: BotStatus::Offline,
      username: username.to_string(),
      uuid: Uuid::nil(),
      connection: None,
      storage: Storage::default(),
      components: BotComponents {
        physics: Physics::default(),
        state: State::default(),
        profile: Profile::default()
      },
      plugins: BotPlugins::default(),
      client_information: ClientInformation {
        particle_status: ParticleStatus::Minimal,
        ..Default::default()
      },
      command_receiver: receiver,
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

  /// Метод запуска бота, который возвращает JoinHandle и не блокирует поток.
  pub fn spawn(mut self, server_host: &str, server_port: u16) -> JoinHandle<io::Result<()>> {
    let host = server_host.to_string();
    let port = server_port;
    
    tokio::spawn(async move {
      self.connect_to(&host, port).await
    })
  }

  /// Метод установки UUID.
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = uuid;
    self
  }

  /// Метод установки информации клиента.
  pub fn set_information(mut self, information: ClientInformation) -> Self {
    self.client_information = information;
    self
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

    self.status = BotStatus::Connecting;

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
    handle_configuration(&mut conn, self.client_information.clone()).await?;

    self.emit_event(BotEvent::ConfigurationFinished);

    let conn = conn.game();
    self.connection = Some(conn);

    self.status = BotStatus::Online;

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
          | ErrorKind::ConnectionAborted 
          | ErrorKind::NotConnected => {
            self.emit_event(BotEvent::Disconnect);

            self.status = BotStatus::Offline;

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
      if self.status != BotStatus::Online {
        return Ok(());
      }

      let Some(conn) = &mut self.connection else {
        continue;
      };

      tokio::select! {
        Ok(packet) = conn.read() => {
          self.emit_event(BotEvent::Packet(&packet));

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
    if let Some(entity_id) = self.components.profile.entity_id {
      if id == entity_id {
        return true;
      }
    }

    false
  }

  /// Метод выполнения определённых операций в каждый физический тик.
  async fn tick(&mut self) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(ErrorKind::NotConnected, format!("Bot {} connection could not be obtained", self.username)));
    };

    if self.plugins.physics.enabled {
      self.components.physics.update(conn).await?;
    }

    Ok(())
  }

  /// Метод очистки данных бота.
  pub fn clear(&mut self) {
    self.storage.entities.clear();
  }

  /// Метод закрытия TcpStream (отключение от сервера).
  pub async fn disconnect(&mut self) -> io::Result<()> {
    let Some(conn) = self.connection.take() else {
      return Err(Error::new(ErrorKind::NotConnected, format!("Bot {} connection could not be obtained", self.username)));
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

    self.clear();

    Ok(())
  }

  /// Метод переподключения бота к серверу.
  pub async fn reconnect(&mut self, server_host: &str, server_port: u16, interval: u64) -> io::Result<()> {
    self.disconnect().await?;
    sleep(interval).await;
    self.connect_to(server_host, server_port).await?;
    Ok(())
  }

  pub async fn chat(&mut self, message: impl Into<String>) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(ErrorKind::NotConnected, format!("Bot {} connection could not be obtained", self.username)));
    };

    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH);
    let timestamp = match duration {
      Ok(d) => d.as_secs(),
      Err(_) => 0,
    };

    conn
      .write(ServerboundGamePacket::Chat(ServerboundChat {
        message: message.into(),
        timestamp: timestamp,
        salt: 0,
        signature: None,
        last_seen_messages: LastSeenMessagesUpdate::default(),
      }))
      .await?;

    Ok(())
  }
}
