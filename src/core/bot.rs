#![allow(dead_code)]

use std::io::{self, Error, ErrorKind};
use std::net::ToSocketAddrs;
use std::sync::Arc;

use azalea_protocol::common::client_information::{ClientInformation, ParticleStatus};
use azalea_protocol::connect::Connection;
use azalea_protocol::packets::game::s_chat::LastSeenMessagesUpdate;
use azalea_protocol::packets::game::{
  ClientboundGamePacket, ServerboundChat, ServerboundGamePacket,
};
use azalea_protocol::packets::handshake::s_intention::ServerboundIntention;
use azalea_protocol::packets::login::s_hello::ServerboundHello;
use azalea_protocol::packets::login::s_login_acknowledged::ServerboundLoginAcknowledged;
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use tokio::io::AsyncWriteExt;
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::core::common::{BotCommand, BotComponents, BotPlugins, BotStatus, BotTerminal};
use crate::core::components::{Physics, Profile, State};
use crate::core::data::{Storage, StorageLock};
use crate::core::events::{BotEvent, EventInvoker, PacketPayload};
use crate::core::handlers::base_processor::{handle_configuration, handle_login};
use crate::core::handlers::command_processor::{CommandProcessorFn, default_command_processor};
use crate::core::handlers::packet_processor::{PacketProcessorFn, default_packet_processor};
use crate::utils::{sleep, timestamp};

pub struct Bot {
  /// Статус подключения бота (offline / connecting / online)
  pub status: BotStatus,

  /// Терминал бота, используется для управления
  pub terminal: Arc<BotTerminal>,

  /// Юзернейм бота
  pub username: String,

  /// UUID бота, по умолчанию нулевой
  pub uuid: Uuid,

  /// Подключение бота в состоянии Play
  pub connection: Option<Connection<ClientboundGamePacket, ServerboundGamePacket>>,

  /// Компоненты бота
  pub components: BotComponents,

  /// Плагины бота и их настройки
  pub plugins: BotPlugins,

  /// Local-хранилище бота
  pub local_storage: StorageLock,

  /// Shared-хранилище бота (опциональное)
  pub shared_storage: Option<StorageLock>,

  client_information: ClientInformation,
  command_receiver: mpsc::Receiver<BotCommand>,
  packet_processor: PacketProcessorFn,
  command_processor: CommandProcessorFn,
  event_invoker: Arc<EventInvoker>,
}

impl Bot {
  pub fn new(username: &str) -> Self {
    let (sender, receiver) = mpsc::channel(50);

    let bot = Self {
      status: BotStatus::Offline,
      terminal: Arc::new(BotTerminal {
        receiver: username.to_string(),
        cmd: sender,
      }),
      username: username.to_string(),
      uuid: Uuid::nil(),
      connection: None,
      local_storage: Arc::new(RwLock::new(Storage::new())),
      shared_storage: None,
      components: BotComponents {
        physics: Physics::default(),
        state: State::default(),
        profile: Profile::default(),
      },
      plugins: BotPlugins::default(),
      client_information: ClientInformation {
        particle_status: ParticleStatus::Minimal,
        ..Default::default()
      },
      command_receiver: receiver,
      packet_processor: default_packet_processor,
      command_processor: default_command_processor,
      event_invoker: Arc::new(EventInvoker::new()),
    };

    bot
  }

  /// Метод запуска бота, который возвращает JoinHandle и не блокирует поток.
  pub fn spawn(mut self, server_host: &str, server_port: u16) -> JoinHandle<io::Result<()>> {
    let host = server_host.to_string();
    let port = server_port;

    tokio::spawn(async move { self.connect_to(&host, port).await })
  }

  /// Метод установки UUID.
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = uuid;
    self
  }

  /// Метод установки информации клиента
  pub fn set_information(mut self, information: ClientInformation) -> Self {
    self.client_information = information;
    self
  }

  /// Метод установки плагинов бота
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }

  /// Метод установки shared-хранилища
  pub fn set_shared_storage(mut self, storage: StorageLock) -> Self {
    self.shared_storage = Some(storage);
    self
  }

  /// Метод установки обработчика пакетов
  pub fn set_packet_processor(mut self, processor: PacketProcessorFn) -> Self {
    self.packet_processor = processor;
    self
  }

  /// Метод установки обработчика команд
  pub fn set_command_processor(mut self, processor: CommandProcessorFn) -> Self {
    self.command_processor = processor;
    self
  }

  /// Метод установки инициатора событий
  pub fn set_event_invoker(mut self, invoker: EventInvoker) -> Self {
    self.event_invoker = Arc::new(invoker);
    self
  }

  /// Метод отправки события
  pub fn emit_event(&self, event: BotEvent) {
    let invoker = Arc::clone(&self.event_invoker);

    let terminal = Arc::clone(&self.terminal);

    tokio::spawn(async move {
      invoker.trigger(terminal, event).await;
    });
  }

  /// Метод создания соединения с сервером и запуска `event_loop`
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

  /// Метод, который подключает бота к серверу, ловит его ошибки и корректно обрабатывает их
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

  /// Метод основного цикла событий
  async fn event_loop(&mut self) -> io::Result<()> {
    let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
      // Проверка текущего статуса подключения.
      // Если статус offline возвращаем Ok(()), чтобы избежать дублированных циклов событий
      if self.status != BotStatus::Online {
        return Ok(());
      }

      let Some(conn) = &mut self.connection else {
        continue;
      };

      tokio::select! {
        // Обработка пакета
        Ok(packet) = conn.read() => {
          self.emit_event(BotEvent::Packet(PacketPayload {
            packet: packet.clone(),
            timestamp: timestamp()
          }));

          match (self.packet_processor)(self, packet).await {
            Ok(true) => continue,
            Ok(false) => return Ok(()),
            Err(e) => return Err(e),
          }
        }

        // Обработка внешней команды
        Some(command) = self.command_receiver.recv() => {
          match (self.command_processor)(self, command).await {
            Ok(true) => continue,
            Ok(false) => return Ok(()),
            Err(e) => return Err(e),
          }
        }

        // Обработка физического тика
        _ = tick_interval.tick() => {
          if let Err(e) = self.tick().await {
            return Err(e);
          }
        }
      }
    }
  }

  /// Метод проверки некого Entity ID на сходство с Entity ID текущего бота
  pub fn is_this_my_entity_id(&self, id: i32) -> bool {
    if let Some(entity_id) = self.components.profile.entity_id {
      if id == entity_id {
        return true;
      }
    }

    false
  }

  /// Метод выполнения определённых операций в каждый физический тик
  async fn tick(&mut self) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(
        ErrorKind::NotConnected,
        format!("Bot {} connection could not be obtained", self.username),
      ));
    };

    if self.plugins.physics.enabled {
      self.components.physics.update(conn).await?;
    }

    Ok(())
  }

  /// Метод очистки данных бота
  pub async fn clear(&mut self) {
    self.local_storage.write().await.entities.clear();
  }

  /// Метод закрытия TcpStream (отключение от сервера)
  pub async fn disconnect(&mut self) -> io::Result<()> {
    let Some(conn) = self.connection.take() else {
      return Err(Error::new(
        ErrorKind::NotConnected,
        format!("Bot {} connection could not be obtained", self.username),
      ));
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

    self.clear().await;

    Ok(())
  }

  /// Метод переподключения бота к серверу
  pub async fn reconnect(
    &mut self,
    server_host: &str,
    server_port: u16,
    interval: u64,
  ) -> io::Result<()> {
    self.disconnect().await?;
    sleep(interval).await;
    self.connect_to(server_host, server_port).await?;
    Ok(())
  }

  pub async fn chat(&mut self, message: impl Into<String>) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(
        ErrorKind::NotConnected,
        format!("Bot {} connection could not be obtained", self.username),
      ));
    };

    conn
      .write(ServerboundGamePacket::Chat(ServerboundChat {
        message: message.into(),
        timestamp: timestamp(),
        salt: 0,
        signature: None,
        last_seen_messages: LastSeenMessagesUpdate::default(),
      }))
      .await?;

    Ok(())
  }
}
