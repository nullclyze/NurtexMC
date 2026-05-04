use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use hashbrown::HashMap;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::bot::connection::spawn_connection;
use crate::bot::handlers::{Handlers};
use crate::bot::plugins::Plugins;
use crate::bot::types::{Connection, PacketReader, PacketWriter};
use crate::bot::{BotComponents, BotProfile, ClientInfo};
use crate::protocol::connection::{ClientsidePacket, NurtexConnection};
use crate::protocol::packets::play::ServersidePlayPacket;
use crate::protocol::types::{Rotation, Vector3};
use crate::proxy::Proxy;
use crate::storage::Storage;
use crate::swarm::Speedometer;
use crate::world::Entity;

/// Структура Minecraft бота
///
/// ## Примеры
/// ```rust, ignore
/// use nurtex::bot::{Bot, BotChatExt};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём бота
///   let mut bot = Bot::create("nurtex_bot");
///
///   // Подключаем бота к серверу
///   bot.connect("localhost", 25565);
///
///   // Ждём немножко
///   tokio::time::sleep(std::time::Duration::from_secs(3)).await;
///
///   // Отправляем сообщение в чат
///   bot.chat_message("Привет, мир!").await?;
///
///   // Ожидаем окончания хэндла подключения
///   bot.wait_handle().await
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/NurtexMC/nurtex/blob/main/crates/nurtex/examples)
pub struct Bot {
  pub profile: Arc<RwLock<BotProfile>>,
  pub connection: Connection,
  handle: Option<JoinHandle<core::result::Result<(), std::io::Error>>>,
  username: String,
  protocol_version: i32,
  connection_timeout: u64,
  reader_tx: PacketReader,
  writer_tx: PacketWriter,
  proxy: Arc<RwLock<Option<Proxy>>>,
  plugins: Arc<Plugins>,
  speedometer: Option<Arc<Speedometer>>,
  components: Arc<RwLock<BotComponents>>,
  storage: Arc<Storage>,
  handlers: Arc<Handlers>,
}

impl Bot {
  /// Метод создания нового бота
  pub fn create(username: impl Into<String>) -> Self {
    Self::create_with_options(username, 45, 45, None, None)
  }

  /// Метод создания нового бота с прокси
  pub fn create_with_proxy(username: impl Into<String>, proxy: Proxy) -> Self {
    Self::create_with_options(username, 45, 45, None, Some(proxy))
  }

  /// Метод создания нового бота со спидометром
  pub fn create_with_speedometer(username: impl Into<String>, speedometer: Arc<Speedometer>) -> Self {
    Self::create_with_options(username, 45, 45, Some(speedometer), None)
  }

  /// Метод создания нового бота с заданными опциями
  pub fn create_with_options(username: impl Into<String>, reader_capacity: usize, writer_capacity: usize, speedometer: Option<Arc<Speedometer>>, proxy: Option<Proxy>) -> Self {
    let (reader_tx, _) = broadcast::channel(reader_capacity);
    let (writer_tx, _) = broadcast::channel(writer_capacity);

    let name = username.into();
    let profile = BotProfile::new(name.clone());

    Self {
      profile: Arc::new(RwLock::new(profile)),
      connection: Arc::new(RwLock::new(None::<NurtexConnection>)),
      plugins: Arc::new(Plugins::default()),
      protocol_version: 774,
      connection_timeout: 14000,
      proxy: Arc::new(RwLock::new(proxy)),
      username: name,
      handle: None,
      reader_tx: Arc::new(reader_tx),
      writer_tx: Arc::new(writer_tx),
      speedometer,
      components: Arc::new(RwLock::new(BotComponents::default())),
      storage: Arc::new(Storage::null()),
      handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при подключении бота)
  pub fn run_reader(connection: Connection, reader_tx: PacketReader) -> JoinHandle<()> {
    tokio::spawn(async move {
      // Может быть гонка условий с NurtexConnection, поэтому небольшая задержка нужна
      tokio::time::sleep(Duration::from_millis(500)).await;

      loop {
        let connected = {
          let conn_guard = connection.read().await;
          conn_guard.is_some()
        };

        if !connected {
          tokio::time::sleep(Duration::from_millis(100)).await;
          continue;
        }

        let packet_result = {
          let conn_guard = connection.read().await;
          if let Some(conn) = conn_guard.as_ref() { conn.try_read_packet() } else { Ok(None) }
        };

        match packet_result {
          Ok(Some(packet)) => {
            if reader_tx.send(packet).is_err() {
              break;
            }
          }
          Ok(None) => tokio::time::sleep(Duration::from_millis(10)).await,
          Err(_) => return,
        }
      }
    })
  }

  /// Метод запуска `writer` (выполняется автоматически при подключении бота)
  pub fn run_writer(connection: Connection, writer_tx: PacketWriter) -> JoinHandle<()> {
    let mut writer_rx = writer_tx.subscribe();

    tokio::spawn(async move {
      // Может быть гонка условий с NurtexConnection, поэтому небольшая задержка нужна
      tokio::time::sleep(Duration::from_millis(800)).await;

      loop {
        if let Ok(packet) = writer_rx.recv().await {
          let conn_guard = connection.read().await;
          if let Some(conn) = conn_guard.as_ref() {
            let _ = conn.write_play_packet(packet).await;
          } else {
            tokio::time::sleep(Duration::from_millis(50)).await;
          }
        }
      }
    })
  }

  /// Метод установки плагинов
  pub fn with_plugins(mut self, plugins: Plugins) -> Self {
    self.plugins = Arc::new(plugins);
    self
  }

  /// Метод установки спидометра
  pub fn with_speedometer(mut self, speedometer: Arc<Speedometer>) -> Self {
    self.speedometer = Some(speedometer);
    self
  }

  /// Метод установки версии протокола
  pub fn with_protocol_version(mut self, protocol_version: i32) -> Self {
    self.protocol_version = protocol_version;
    self
  }

  /// Метод установки таймаута подключения
  pub fn with_connection_timeout(mut self, timeout: u64) -> Self {
    self.connection_timeout = timeout;
    self
  }

  /// Метод установки прокси
  pub fn with_proxy(mut self, proxy: Proxy) -> Self {
    self.proxy = Arc::new(RwLock::new(Some(proxy)));
    self
  }

  /// Метод установки информации клиента
  pub fn with_information(self, information: ClientInfo) -> Self {
    // Здесь почти невозможен исход с ошибкой, поэтому просто игнорируем
    match self.profile.try_write() {
      Ok(mut g) => g.information = information,
      Err(_) => {}
    }

    self
  }

  /// Метод установки обработчиков
  pub fn with_handlers(mut self, handlers: Handlers) -> Self {
    self.handlers = Arc::new(handlers);
    self
  }

  /// Метод установки общего хранилища
  pub fn set_shared_storage(mut self, storage: Arc<Storage>) -> Self {
    self.storage = storage;
    self
  }

  /// Метод установки общих обработчиков
  pub fn set_shared_handlers(mut self, handlers: Arc<Handlers>) -> Self {
    self.handlers = handlers;
    self
  }

  /// Метод получения юзернейма
  pub fn username(&self) -> &str {
    &self.username
  }

  /// Метод получения UUID
  pub async fn uuid(&self) -> Uuid {
    self.profile.read().await.uuid
  }

  /// Метод получения профиля бота
  pub fn get_profile(&self) -> Arc<RwLock<BotProfile>> {
    Arc::clone(&self.profile)
  }

  /// Метод получения прокси бота
  pub fn get_proxy(&self) -> Arc<RwLock<Option<Proxy>>> {
    Arc::clone(&self.proxy)
  }

  /// Метод получения хранилища
  pub fn get_storage(&self) -> Arc<Storage> {
    Arc::clone(&self.storage)
  }

  /// Вспомогательный метод подписки на слушание пакетов
  pub fn subscribe_to_reader(&self) -> broadcast::Receiver<ClientsidePacket> {
    self.reader_tx.subscribe()
  }

  /// Метод получения копии `reader_tx`
  pub fn get_reader(&self) -> PacketReader {
    Arc::clone(&self.reader_tx)
  }

  /// Метод получения копии `writer_tx`
  pub fn get_writer(&self) -> PacketWriter {
    Arc::clone(&self.writer_tx)
  }

  /// Метод получения копии подключения
  pub fn get_connection(&self) -> Connection {
    Arc::clone(&self.connection)
  }

  /// Метод получения хэндла
  pub fn get_handle(&self) -> &Option<JoinHandle<core::result::Result<(), std::io::Error>>> {
    &self.handle
  }

  /// Метод отправки пакета
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer_tx.send(packet);
  }

  /// Метод подключения бота к серверу.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use nurtex::Bot;
  ///
  /// #[tokio::main]
  /// async fn main() -> std::io::Result<()> {
  ///   // Создаём бота
  ///   let mut bot = Bot::create("nurtex_bot");
  ///
  ///   // Подключаем бота к серверу.
  ///   // Если после вызова этого метода выполняются
  ///   // какие-либо действия с ботом, рекомендуется
  ///   // подождать несколько секунд, чтобы бот
  ///   // полностью подключился к серверу
  ///   bot.connect("localhost", 25565);
  ///
  ///   // Ожидаем окончания хэндла подключения
  ///   bot.wait_handle().await
  /// }
  /// ```
  pub fn connect(&mut self, server_host: impl Into<String>, server_port: u16) {
    self.handle = Some(self.connect_with_handle(server_host, server_port));
  }

  /// Метод подключения бота к серверу, возвращающий хэндл подключения
  pub fn connect_with_handle(&self, server_host: impl Into<String>, server_port: u16) -> JoinHandle<Result<(), std::io::Error>> {
    let connection = Arc::clone(&self.connection);
    let profile = Arc::clone(&self.profile);
    let components = Arc::clone(&self.components);
    let speedometer = self.speedometer.clone();
    let plugins = Arc::clone(&self.plugins);
    let proxy = Arc::clone(&self.proxy);
    let reader_tx = Arc::clone(&self.reader_tx);
    let writer_tx = Arc::clone(&self.writer_tx);
    let storage = Arc::clone(&self.storage);
    let handlers = Arc::clone(&self.handlers);
    let protocol_version = self.protocol_version;
    let coonnection_timeout = self.connection_timeout;
    let host = server_host.into();
    let port = server_port;

    tokio::spawn(async move {
      let mut reconnection_attempts = 0;
      let max_attempts = if plugins.auto_reconnect.enabled { plugins.auto_reconnect.max_attempts } else { 1 };

      loop {
        let reader_handle = Self::run_reader(Arc::clone(&connection), Arc::clone(&reader_tx));
        let writer_handle = Self::run_writer(Arc::clone(&connection), Arc::clone(&writer_tx));

        let result = spawn_connection(
          &connection,
          &profile,
          &components,
          &speedometer,
          &plugins,
          &reader_tx,
          &storage,
          protocol_version,
          coonnection_timeout,
          &proxy,
          &host,
          port,
          &handlers,
        )
        .await;

        // На этом моменте бот считается не подключенным к серверу, поэтому нужно отменять reader / writer
        reader_handle.abort();
        writer_handle.abort();

        match result {
          Ok(_) => return Ok(()),
          Err(e) => match e.kind() {
            ErrorKind::ConnectionAborted | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::TimedOut | ErrorKind::NotConnected | ErrorKind::NetworkDown => {
              if !plugins.auto_reconnect.enabled || (max_attempts != -1 && reconnection_attempts >= max_attempts) {
                return Err(e);
              }

              reconnection_attempts += 1;

              tokio::time::sleep(Duration::from_millis(plugins.auto_reconnect.reconnect_delay)).await;
            }
            _ => return Err(e),
          },
        }
      }
    })
  }

  /// Метод ожидания завершения хэндла подключения
  pub async fn wait_handle(&mut self) -> std::io::Result<()> {
    if let Some(handle) = self.handle.as_mut() { handle.await? } else { Ok(()) }
  }

  /// Метод полноценной очистки и отключения бота
  pub async fn shutdown(&self) -> std::io::Result<()> {
    self.abort_handle();

    let mut conn_guard = self.connection.write().await;
    if let Some(conn) = conn_guard.as_ref() {
      conn.shutdown().await?;
    }

    *conn_guard = None;
    std::mem::drop(conn_guard);

    self.clear().await;

    Ok(())
  }

  /// Метод очистки данных бота
  pub async fn clear(&self) {
    *self.components.write().await = BotComponents::default();
  }

  /// Метод отмены хэндла бота
  pub fn abort_handle(&self) {
    if let Some(handle) = &self.handle {
      handle.abort();
    }
  }

  /// Метод переподключения бота
  pub async fn reconnect(&mut self, server_host: impl Into<String>, server_port: u16, reconnect_delay: u64) -> std::io::Result<()> {
    self.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
    self.connect(server_host, server_port);
    Ok(())
  }

  /// Метод переподключения бота, возвращающий хэндл подключения
  pub async fn reconnect_with_handle(&mut self, server_host: impl Into<String>, server_port: u16, reconnect_delay: u64) -> std::io::Result<JoinHandle<Result<(), std::io::Error>>> {
    self.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
    Ok(self.connect_with_handle(server_host, server_port))
  }

  /// Метод получения компонентов бота
  pub fn get_components(&self) -> Arc<RwLock<BotComponents>> {
    Arc::clone(&self.components)
  }

  /// Метод получения опциональной позиции бота
  pub fn try_get_position(&self) -> Option<Vector3> {
    match self.components.try_read() {
      Ok(g) => Some(g.position.clone()),
      Err(_) => None,
    }
  }

  /// Метод получения опционального здоровья бота
  pub fn try_get_health(&self) -> Option<Vector3> {
    match self.components.try_read() {
      Ok(g) => Some(g.position.clone()),
      Err(_) => None,
    }
  }

  /// Метод получения опциональной ротации бота
  pub fn try_get_rotation(&self) -> Option<f32> {
    match self.components.try_read() {
      Ok(g) => Some(g.health),
      Err(_) => None,
    }
  }

  /// Метод получения опциональных сущностей из хранилища
  pub fn try_get_entities(&self) -> Option<HashMap<i32, Entity>> {
    match self.storage.entities.try_read() {
      Ok(g) => {
        let mut entities = HashMap::with_capacity(g.len());

        for (entity_id, entity) in &*g {
          entities.insert(*entity_id, entity.clone());
        }

        Some(entities)
      }
      Err(_) => None,
    }
  }

  /// Метод получения позиции бота
  pub async fn get_position(&self) -> Vector3 {
    let guard = self.components.read().await;
    guard.position.clone()
  }

  /// Метод получения ротации бота
  pub async fn get_rotation(&self) -> Rotation {
    let guard = self.components.read().await;
    guard.rotation.clone()
  }

  /// Метод получения здоровья бота
  pub async fn get_health(&self) -> f32 {
    let guard = self.components.read().await;
    guard.health
  }

  /// Метод получения всех сущностей из хранилища
  pub async fn get_entities(&self) -> HashMap<i32, Entity> {
    let guard = self.storage.entities.read().await;
    let mut entities = HashMap::with_capacity(guard.len());

    for (entity_id, entity) in &*guard {
      entities.insert(*entity_id, entity.clone());
    }

    entities
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use crate::bot::handlers::Handlers;
  use crate::protocol::connection::ClientsidePacket;
  use crate::protocol::packets::play::ClientsidePlayPacket;
  use crate::proxy::Proxy;

  use crate::bot::plugins::{AutoReconnectPlugin, AutoRespawnPlugin, Plugins};
  use crate::bot::{Bot, BotChatExt};

  #[tokio::test]
  async fn test_packet_handling() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);

        // + Доп проверка взаимодействия с чатом

        match packet {
          ClientsidePlayPacket::KeepAlive(p) => {
            bot.chat_message(format!("Получен KeepAlive: {}", p.id)).await?;
          }
          _ => {}
        }
      }
    }
  }

  #[tokio::test]
  async fn test_handlers() -> io::Result<()> {
    let mut handlers = Handlers::new();

    handlers.on_login(async |username| {
      println!("Бот {} залогинился", username);
      Ok(())
    });

    handlers.on_spawn(async |username| {
      println!("Бот {} заспавнился", username);
      Ok(())
    });

    handlers.on_chat(async |username, payload| {
      println!("Бот {} получил сообщение: {}", username, payload.message);
      Ok(())
    });

    handlers.on_disconnect(async |username, payload| {
      println!("Бот {} отключился в состоянии: {:?}", username, payload.state);
      Ok(())
    });

    let mut bot = Bot::create("nurtex_bot").with_handlers(handlers);

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }

  #[tokio::test]
  async fn test_auto_respawn() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").with_plugins(Plugins {
      auto_respawn: AutoRespawnPlugin {
        enabled: true,
        respawn_delay: 2000,
      },
      ..Default::default()
    });

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }

  #[tokio::test]
  async fn test_auto_reconnect() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").with_plugins(Plugins {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true,
        reconnect_delay: 1000,
        max_attempts: 3,
      },
      ..Default::default()
    });

    bot.connect("localhost", 25565);

    // + Доп проверка на работоспособность reader'а пакетов после переподключения

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_entity_storage() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    tokio::time::sleep(Duration::from_secs(3)).await;

    for _ in 0..10 {
      println!("Сущности: {:?}", bot.get_entities().await);
      tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
  }

  #[tokio::test]
  async fn test_bot_with_socks5_proxy() -> io::Result<()> {
    let proxy = Proxy::from("socks5://212.58.132.5:1080");
    let mut bot = Bot::create_with_proxy("l7jqw8d5", proxy);

    bot.connect("hub.holyworld.ru", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_bot_with_socks4_proxy() -> io::Result<()> {
    let proxy = Proxy::from("socks4://68.71.242.118:4145");
    let mut bot = Bot::create_with_proxy("k72ido3d", proxy);

    bot.connect("hub.holyworld.ru", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_reconnect() -> io::Result<()> {
    let mut handlers = Handlers::new();

    handlers.on_spawn(async |username| {
      println!("Бот {} заспавнился", username);
      Ok(())
    });

    let mut bot = Bot::create("nurtex_bot").with_handlers(handlers);

    let server_host = "localhost".to_string();
    let server_port = 25565;

    bot.connect(&server_host, server_port);

    tokio::time::sleep(Duration::from_secs(3)).await;

    bot.reconnect(&server_host, server_port, 1000).await?;

    bot.wait_handle().await
  }
}
