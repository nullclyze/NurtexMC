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
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::bot::account::BotAccount;
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
use crate::bot::world::{Storage, StorageLock};
use crate::utils::time::{sleep, timestamp};

/// Основная структура бота, состояния, компоненты, методы подключения -
/// всё хранится именно здесь. Данный объект содержит в себе всю
/// информацию о мире (`Storage`), но при запуске роя бот будет отправлять
/// все данные в `SharedStorage` (если в `Swarm` значение у
/// флага `use_shared_storage` является `true` для этого бота).
///
/// Generic параметр `B` определяет тип bundle'а данных, который будет
/// передаваться через `BotTransmitter`. По умолчанию используется `StandardBundle`.
///
/// Пример создания и подключения бота к серверу:
/// ```rust, ignore
/// // Создаём бота с стандартным bundle'ом
/// let account = BotAccount::new("NurtexBot");
/// let mut bot = Bot::create(account);
///
/// // Или с пользовательским bundle'ом
/// let mut bot = Bot::<MyCustomBundle>::create(account);
///
/// // Подключаем бота к серверу
/// bot.connect_to("server.com", 25565).await?;
/// ```
pub struct Bot<P: BotPackage = NullPackage> {
  /// Статус подключения бота
  pub status: BotStatus,

  /// Терминал бота, используется для управления
  pub terminal: Arc<BotTerminal>,

  /// Аккаунт бота
  pub account: Arc<BotAccount>,

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

  transmitter: BotTransmitter<P>,
  transmitter_interval: u64,
  connection_timeout: u64,
  proxy: Option<Proxy>,
  information: BotInformation,
  command_receiver: mpsc::Receiver<BotCommand>,
  packet_processor: PacketProcessorFn<P>,
  command_processor: CommandProcessorFn<P>,
  event_invoker: Arc<EventInvoker>,
}

impl<P: BotPackage> Bot<P> {
  /// Метод создания нового бота.
  ///
  /// Пример создания и базовой настройки:
  /// ```rust, ignore
  /// // Создаём аккаунт
  /// let account = BotAccount::new("NurtexBot");
  ///  
  /// // Создаём и настраиваем бота
  /// let mut bot = Bot::create(account)
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
  pub fn create(account: BotAccount) -> Self {
    let (sender, receiver) = mpsc::channel(25);

    let bot_account = Arc::new(account);

    Self {
      status: BotStatus::Offline,
      terminal: Arc::new(BotTerminal {
        account: Arc::clone(&bot_account),
        cmd: sender,
      }),
      account: bot_account,
      connection: None,
      local_storage: Arc::new(RwLock::new(Storage::new())),
      shared_storage: None,
      components: BotComponents::default(),
      plugins: BotPlugins::default(),
      information: BotInformation::default(),
      physics: Physics::default(),
      transmitter: BotTransmitter::new(10),
      transmitter_interval: 100,
      connection_timeout: 14000,
      proxy: None,
      command_receiver: receiver,
      packet_processor: default_packet_processor,
      command_processor: default_command_processor,
      event_invoker: Arc::new(EventInvoker::new()),
    }
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

  /// Метод отправки события
  pub fn emit_event(&self, event: BotEvent) {
    let invoker = Arc::clone(&self.event_invoker);
    let terminal = Arc::clone(&self.terminal);

    tokio::spawn(async move {
      invoker.trigger(terminal, event).await;
    });
  }

  /// Метод получения ссылки на информацию бота
  pub fn get_information_ref(&self) -> &BotInformation {
    &self.information
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
        name: self.account.username.clone(),
        profile_id: self.account.uuid,
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
    }

    Ok(())
  }

  /// Метод очистки данных бота
  pub async fn clear(&mut self) {
    let mut storage = self.local_storage.write().await;
    storage.entities.clear();
    storage.entities.shrink_to_fit();
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
    self.connect_to(server_host, server_port).await?;
    Ok(())
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
      .await?;

    Ok(())
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

  /// Метод отправки пакета данных
  fn emit_package(&self) {
    if self.transmitter.receiver_count() < 1 {
      return;
    }

    let package = P::describe(self);

    self.transmitter.emit(package);
  }

  /// Метод получения передатчика пакетов
  pub fn get_transmitter(&self) -> BotTransmitter<P> {
    self.transmitter.clone()
  }
}
