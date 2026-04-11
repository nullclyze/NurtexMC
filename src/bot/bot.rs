#![allow(dead_code)]

use std::io::{self, Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use azalea_protocol::connect::{Connection, Proxy};
use azalea_protocol::packets::game::s_chat::LastSeenMessagesUpdate;
use azalea_protocol::packets::game::{ClientboundGamePacket, ServerboundChat, ServerboundGamePacket};
use azalea_protocol::packets::handshake::s_intention::ServerboundIntention;
use azalea_protocol::packets::handshake::{ClientboundHandshakePacket, ServerboundHandshakePacket};
use azalea_protocol::packets::login::s_hello::ServerboundHello;
use azalea_protocol::packets::login::s_login_acknowledged::ServerboundLoginAcknowledged;
use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};
use tokio::io::AsyncWriteExt;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

use crate::bot::components::BotComponents;
use crate::bot::components::position::Position;
use crate::bot::components::rotation::Rotation;
use crate::bot::events::{BotEvent, EventInvoker, PacketPayload, RotationPayload};
use crate::bot::events::{DisconnectPayload, PositionPayload};
use crate::bot::handlers::base::{process_configuration, process_login};
use crate::bot::handlers::custom::command::{CommandProcessorFn, default_command_processor};
use crate::bot::handlers::custom::packet::{PacketProcessorFn, default_packet_processor};
use crate::bot::options::{BotInformation, BotPlugins, BotStatus};
use crate::bot::physics::Physics;
use crate::bot::terminal::{BotCommand, BotTerminal};
use crate::bot::transmitter::{BotPackage, BotTransmitter, NullPackage};
use crate::bot::world::entity::Entity;
use crate::bot::world::{Storage, StorageLock};
use crate::utils::time::{sleep, timestamp};

/// Основная структура бота, состояния, компоненты, методы подключения -
/// всё хранится именно здесь. Данный объект содержит в себе всю
/// информацию о мире (`Storage`), но при запуске роя бот будет отправлять
/// все данные в `SharedStorage` (если в `Swarm` значение у
/// флага `use_shared_storage` является `true` для этого бота).
///
/// Пример создания и подключения бота к серверу:
/// ```rust, ignore
/// // Создаём бота
/// let mut bot = create_bot("NurtexBot");
///
/// // Подключаем бота к серверу
/// bot.connect_to("server.com", 25565).await?;
/// ```
pub struct Bot<P: BotPackage = NullPackage> {
  /// Статус подключения бота
  pub status: BotStatus,

  /// Юзернейм бота
  pub username: String,

  /// UUID бота
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

  /// Физика бота
  pub physics: Physics,

  terminal: Arc<BotTerminal>,
  transmitter: Arc<BotTransmitter<P>>,
  transmitter_interval: u64,
  connection_timeout: u64,
  proxy: Option<Proxy>,
  information: BotInformation,
  event_sender: broadcast::Sender<BotEvent>,
  command_receiver: mpsc::Receiver<BotCommand>,
  packet_processor: PacketProcessorFn<P>,
  command_processor: CommandProcessorFn<P>,
  event_invoker: Arc<EventInvoker>,
}

impl<P: BotPackage> Bot<P> {
  /// Метод создания нового бота.
  ///
  /// Пример использования:
  /// ```rust, ignore  
  /// // Создаём и настраиваем бота
  /// let mut bot: Bot<NullPackage> = Bot::create("NurtexBot".to_string())
  ///   .set_connection_timeout(30000)
  ///   .set_information(BotInformation {
  ///     client: ClientInformation {
  ///       main_hand: HumanoidArm::Left,
  ///       ..Default::default()
  ///     },
  ///     ..Default::default(),
  ///   });
  ///
  /// // Подключаем бота к серверу
  /// bot.connect_to("server.com", 25565).await?;
  /// ```
  pub fn create(username: String) -> Self {
    let (event_tx, _) = broadcast::channel(30);
    let (command_tx, command_rx) = mpsc::channel(30);

    let bot = Self {
      status: BotStatus::Offline,
      terminal: Arc::new(BotTerminal {
        username: username.clone(),
        cmd: command_tx,
      }),
      username: username,
      uuid: Uuid::nil(),
      connection: None,
      local_storage: Arc::new(RwLock::new(Storage::new())),
      shared_storage: None,
      components: BotComponents::default(),
      plugins: BotPlugins::default(),
      information: BotInformation::default(),
      physics: Physics::default(),
      transmitter: Arc::new(BotTransmitter::new(30)),
      transmitter_interval: 500,
      connection_timeout: 14000,
      proxy: None,
      event_sender: event_tx,
      command_receiver: command_rx,
      packet_processor: default_packet_processor,
      command_processor: default_command_processor,
      event_invoker: Arc::new(EventInvoker::new()),
    };

    bot.activate_invoker();

    bot
  }

  /// Метод запуска бота, который возвращает JoinHandle и не блокирует поток.
  pub fn spawn(mut self, server_host: &str, server_port: u16) -> JoinHandle<io::Result<()>> {
    let host = server_host.to_string();

    tokio::spawn(async move { self.connect_to(&host, server_port).await })
  }

  /// Метод установки таймаута подключения
  pub fn set_connection_timeout(mut self, timeout: u64) -> Self {
    self.connection_timeout = timeout;
    self
  }

  /// Метод установки прокси
  pub fn set_proxy(mut self, proxy: Proxy) -> Self {
    self.proxy = Some(proxy);
    self
  }

  /// Метод установки UUID
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = uuid;
    self
  }

  /// Метод установки информации бота
  pub fn set_information(mut self, information: BotInformation) -> Self {
    self.information = information;
    self
  }

  /// Метод установки плагинов бота
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }

  /// Метод установки интервала передатчика
  pub fn set_transmitter_interval(mut self, interval: u64) -> Self {
    self.transmitter_interval = interval;
    self
  }

  /// Метод установки shared-хранилища
  pub fn set_shared_storage(mut self, storage: StorageLock) -> Self {
    self.shared_storage = Some(storage);
    self
  }

  /// Метод установки обработчика пакетов
  pub fn set_packet_processor(mut self, processor: PacketProcessorFn<P>) -> Self {
    self.packet_processor = processor;
    self
  }

  /// Метод установки обработчика команд
  pub fn set_command_processor(mut self, processor: CommandProcessorFn<P>) -> Self {
    self.command_processor = processor;
    self
  }

  /// Метод установки инициатора событий
  pub fn set_event_invoker(mut self, invoker: EventInvoker) -> Self {
    self.event_invoker = Arc::new(invoker);
    self
  }

  /// Метод получения ссылки на информацию бота
  pub fn get_information_ref(&self) -> &BotInformation {
    &self.information
  }

  /// Метод активации инициатора событий
  fn activate_invoker(&self) {
    let invoker = self.event_invoker.clone();
    let terminal = self.terminal.clone();
    let mut receiver = self.event_sender.subscribe();

    tokio::spawn(async move {
      while let Ok(event) = receiver.recv().await {
        invoker.trigger(terminal.clone(), event).await;
      }
    });
  }

  /// Метод отправки события всем получателям
  pub fn emit_event(&self, event: BotEvent) {
    let _ = self.event_sender.send(event);
  }

  /// Метод создания подключения
  async fn create_connection(&self, address: SocketAddr) -> io::Result<Connection<ClientboundHandshakePacket, ServerboundHandshakePacket>> {
    let result = if let Some(proxy) = &self.proxy {
      Connection::new_with_proxy(&address, proxy.clone()).await
    } else {
      Connection::new(&address).await
    };

    result.map_err(|err| Error::new(ErrorKind::ConnectionRefused, err.to_string()))
  }

  /// Метод создания соединения с сервером и запуска цикла событий
  async fn start(&mut self, server_host: &str, server_port: u16) -> io::Result<()> {
    self.connection = None;

    let address_string = format!("{}:{}", server_host, server_port);

    let Some(address) = (match address_string.to_socket_addrs() {
      Ok(mut i) => i.next(),
      Err(err) => return Err(err),
    }) else {
      return Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "Failed to retrieve socket address"));
    };

    let connection_result = timeout(Duration::from_millis(self.connection_timeout), self.perform_connection(address, server_host, server_port)).await;

    match connection_result {
      Ok(Ok(conn)) => {
        self.connection = Some(conn);
        self.status = BotStatus::Online;
        self.event_loop().await?;
        Ok(())
      }
      Ok(Err(err)) => Err(err),
      Err(_) => Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("Failed to get a response from the server within the timeout period ({} ms)", self.connection_timeout),
      )),
    }
  }

  /// Метод полного процесса подключения бота к серверу
  async fn perform_connection(&mut self, address: SocketAddr, server_host: &str, server_port: u16) -> io::Result<Connection<ClientboundGamePacket, ServerboundGamePacket>> {
    let mut conn = self.create_connection(address).await?;

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

    process_login(&mut conn).await?;
    conn.write(ServerboundLoginAcknowledged {}).await?;

    self.emit_event(BotEvent::LoginFinished);

    let mut conn = conn.config();
    process_configuration(self, &mut conn).await?;

    self.emit_event(BotEvent::ConfigurationFinished);

    let conn = conn.game();

    Ok(conn)
  }

  /// Метод, который подключает бота к серверу, ловит его ошибки и корректно обрабатывает их
  pub async fn connect_to(&mut self, server_host: &str, server_port: u16) -> io::Result<()> {
    loop {
      match self.start(server_host, server_port).await {
        Ok(_) => {
          break;
        }
        Err(err) => match err.kind() {
          ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::NotConnected | ErrorKind::TimedOut => {
            self.emit_event(BotEvent::Disconnect(DisconnectPayload {
              reason: err.to_string(),
              timestamp: timestamp(),
            }));

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

    let mut transmitter_interval = tokio::time::interval(tokio::time::Duration::from_millis(self.transmitter_interval));
    transmitter_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
      if self.status != BotStatus::Online {
        return Ok(());
      }

      let Some(conn) = &mut self.connection else {
        continue;
      };

      tokio::select! {
        // Обработка пакета
        result = conn.read() => {
          match result {
            Ok(p) => {
              let packet: Arc<ClientboundGamePacket> = Arc::new(p);

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
            Err(_) => {}
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

        // Обработка тика передатчика
        _ = transmitter_interval.tick() => {
          self.emit_package();
        }
      }
    }
  }

  /// Метод проверки некого Entity ID на сходство с Entity ID текущего бота
  pub fn is_this_my_entity_id(&self, id: i32) -> bool {
    self.components.profile.entity_id.map_or(false, |entity_id| entity_id == id)
  }

  /// Метод выполнения определённых операций в физический тик
  async fn tick(&mut self) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
    };

    if self.plugins.physics.enabled {
      let storage = self.local_storage.read().await;
      self.components.tick(conn, &mut self.physics, &storage).await?;
      drop(storage);
    }

    Ok(())
  }

  /// Метод блокировки хранилища (local / shared)
  pub async fn lock_storage<F>(&self, f: F)
  where
    F: FnOnce(&mut Storage),
  {
    if let Some(shared_storage) = &self.shared_storage {
      if let Ok(mut guard) = timeout(Duration::from_millis(10), shared_storage.write()).await {
        f(&mut *guard);
        drop(guard);
      }
    } else {
      if let Ok(mut guard) = timeout(Duration::from_millis(10), self.local_storage.write()).await {
        f(&mut *guard);
        drop(guard);
      }
    }
  }

  /// Метод очистки данных бота
  pub async fn clear(&mut self) {
    let mut storage = self.local_storage.write().await;
    storage.clear();
  }

  /// Метод закрытия TcpStream (отключение от сервера)
  pub async fn disconnect(&mut self) -> io::Result<()> {
    let Some(conn) = self.connection.take() else {
      return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
    };

    let mut stream = match conn.unwrap() {
      Ok(s) => s,
      Err(err) => {
        return Err(Error::new(ErrorKind::Other, err.to_string()));
      }
    };

    stream.shutdown().await?;

    self.clear().await;

    self.emit_event(BotEvent::Disconnect(DisconnectPayload {
      reason: "Manual disconnect".to_string(),
      timestamp: timestamp(),
    }));

    Ok(())
  }

  /// Метод переподключения бота к серверу
  pub async fn reconnect(&mut self, server_host: &str, server_port: u16, interval: u64) -> io::Result<()> {
    self.disconnect().await?;
    sleep(interval).await;
    self.connect_to(server_host, server_port).await
  }

  /// Метод отправки сообщения в чат
  pub async fn chat(&mut self, message: impl Into<String>) -> io::Result<()> {
    let Some(conn) = &mut self.connection else {
      return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
    };

    conn
      .write(ServerboundGamePacket::Chat(ServerboundChat {
        message: message.into(),
        timestamp: timestamp(),
        salt: 0,
        signature: None,
        last_seen_messages: LastSeenMessagesUpdate::default(),
      }))
      .await
  }

  /// Метод обновления позиции
  pub fn update_position(&mut self, position: Position) {
    let pos = &mut self.components.position;
    let old_pos = pos.clone();

    if old_pos == position {
      return;
    }

    *pos = position.clone();

    self.emit_event(BotEvent::UpdatePosition(PositionPayload {
      position: position,
      old_position: old_pos,
      timestamp: timestamp(),
    }));
  }

  /// Метод обновления ротации
  pub fn update_rotation(&mut self, rotation: Rotation) {
    let rot = &mut self.components.rotation;
    let old_rot = rot.clone();

    if old_rot == rotation {
      return;
    }

    *rot = rotation.clone();

    self.emit_event(BotEvent::UpdateRotation(RotationPayload {
      rotation: rotation,
      old_rotation: old_rot,
      timestamp: timestamp(),
    }));
  }

  /// Метод получения текущей позиции
  pub fn get_position(&self) -> Position {
    self.components.position
  }

  /// Метод получения текущей ротации
  pub fn get_rotation(&self) -> Rotation {
    self.components.rotation
  }

  /// Метод получения сущности игрока по его юзернейму
  pub async fn get_player_by_username(&self, username: String) -> Option<Entity> {
    let mut entity = None;

    self
      .lock_storage(|storage| {
        for (_, e) in &storage.entities {
          let Some(player_info) = &e.player_info else {
            continue;
          };

          if player_info.username == username {
            entity = Some(e.clone());
            return;
          }
        }
      })
      .await;

    entity
  }

  /// Метод получения сущности игрока по его юзернейму
  pub async fn get_entity_by_uuid(&self, uuid: Uuid) -> Option<Entity> {
    let mut entity = None;

    self
      .lock_storage(|storage| {
        for (_, e) in &storage.entities {
          if e.uuid == uuid {
            entity = Some(e.clone());
            return;
          }
        }
      })
      .await;

    entity
  }

  /// Метод отправки пакета данных
  fn emit_package(&self) {
    if self.transmitter.receiver_count() < 1 {
      return;
    }

    let package = P::describe(self);

    self.transmitter.emit(package);
  }

  /// Метод получения терминала
  pub fn get_terminal(&self) -> Arc<BotTerminal> {
    self.terminal.clone()
  }

  /// Метод получения передатчика пакетов
  pub fn get_transmitter(&self) -> Arc<BotTransmitter<P>> {
    self.transmitter.clone()
  }
}
