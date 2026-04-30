use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use hashbrown::HashMap;
use nurtex_protocol::connection::utils::handle_encryption_request;
use nurtex_protocol::connection::{ClientsidePacket, ConnectionState, NurtexConnection};
use nurtex_protocol::packets::play::{ClientsidePlayPacket, ServersideAcceptTeleportation, ServersideClientCommand};
use nurtex_protocol::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks},
  handshake::{ServersideGreet, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart},
  play::ServersidePlayPacket,
};
use nurtex_protocol::types::{ClientCommand, ClientIntention, ResourcePackState, Rotation, Vector3};
use nurtex_proxy::Proxy;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

use crate::bot::capture::{capture_components, capture_connection};
use crate::bot::plugins::BotPlugins;
use crate::bot::{BotComponents, BotProfile, ClientInfo, capture_storage};
use crate::storage::Storage;
use crate::swarm::Speedometer;
use crate::world::Entity;

/// Тип потокобезопасного подключения
pub type BotConnection = Arc<RwLock<Option<NurtexConnection>>>;

/// Тип потокобезопасного `reader`
pub type PacketReader = Arc<broadcast::Sender<ClientsidePacket>>;

/// Тип потокобезопасного `writer`
pub type PacketWriter = Arc<broadcast::Sender<ServersidePlayPacket>>;

/// Структура Minecraft бота
pub struct Bot {
  pub profile: Arc<RwLock<BotProfile>>,
  pub connection: BotConnection,
  protocol_version: i32,
  connection_timeout: u64,
  proxy: Arc<RwLock<Option<Proxy>>>,
  plugins: BotPlugins,
  username: String,
  handle: Option<JoinHandle<core::result::Result<(), std::io::Error>>>,
  reader_tx: PacketReader,
  writer_tx: PacketWriter,
  speedometer: Option<Arc<Speedometer>>,
  components: Arc<RwLock<BotComponents>>,
  storage: Arc<RwLock<Storage>>,
}

impl Bot {
  /// Метод создания нового бота
  pub fn create(username: impl Into<String>) -> Self {
    Self::create_with_options(username, 45, 45, None, None)
  }

  /// Метод создания нового бота
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
      plugins: BotPlugins::default(),
      protocol_version: 774,
      connection_timeout: 14000,
      proxy: Arc::new(RwLock::new(proxy)),
      username: name,
      handle: None,
      reader_tx: Arc::new(reader_tx),
      writer_tx: Arc::new(writer_tx),
      speedometer,
      components: Arc::new(RwLock::new(BotComponents::default())),
      storage: Arc::new(RwLock::new(Storage::null())),
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при подключении бота)
  pub fn run_reader(connection: BotConnection, reader_tx: PacketReader) -> JoinHandle<()> {
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
  pub fn run_writer(connection: BotConnection, writer_tx: PacketWriter) -> JoinHandle<()> {
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

  /// Метод установки информации клиента
  pub async fn set_information(&self, information: ClientInfo) {
    self.profile.write().await.information = information;
  }

  /// Метод установки плагинов
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }

  /// Метод установки спидометра
  pub fn set_speedometer(mut self, speedometer: Arc<Speedometer>) -> Self {
    self.speedometer = Some(speedometer);
    self
  }

  /// Метод установки версии протокола
  pub fn set_protocol_version(mut self, protocol_version: i32) -> Self {
    self.protocol_version = protocol_version;
    self
  }

  /// Метод установки таймаута подключения
  pub fn set_connection_timeout(mut self, timeout: u64) -> Self {
    self.connection_timeout = timeout;
    self
  }

  /// Метод установки SOCKS5 прокси
  pub fn set_proxy(mut self, proxy: Proxy) -> Self {
    self.proxy = Arc::new(RwLock::new(Some(proxy)));
    self
  }

  /// Метод установки хранилища данных
  pub fn set_storage(mut self, storage: Arc<RwLock<Storage>>) -> Self {
    self.storage = storage;
    self
  }

  /// Метод получения юзернейма
  pub fn username(&self) -> &str {
    &self.username
  }

  /// Метод получения профиля бота
  pub fn get_profile(&self) -> Arc<RwLock<BotProfile>> {
    Arc::clone(&self.profile)
  }

  /// Метод получения прокси бота
  pub fn get_proxy(&self) -> Arc<RwLock<Option<Proxy>>> {
    Arc::clone(&self.proxy)
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
  pub fn get_connection(&self) -> BotConnection {
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

  /// Метод подключения бота к серверу
  pub fn connect(&mut self, server_host: impl Into<String>, server_port: u16) {
    self.handle = Some(self.connect_with_handle(server_host, server_port));
  }

  /// Метод подключения бота к серверу, который возвращает хендл бота
  pub fn connect_with_handle(&self, server_host: impl Into<String>, server_port: u16) -> JoinHandle<core::result::Result<(), std::io::Error>> {
    let connection = Arc::clone(&self.connection);
    let profile = Arc::clone(&self.profile);
    let components = Arc::clone(&self.components);
    let speedometer = self.speedometer.clone();
    let plugins = self.plugins.clone();
    let coonnection_timeout = self.connection_timeout;
    let proxy = Arc::clone(&self.proxy);
    let reader_tx = Arc::clone(&self.reader_tx);
    let writer_tx = Arc::clone(&self.writer_tx);
    let storage = Arc::clone(&self.storage);
    let protocol_version = self.protocol_version;
    let host = server_host.into();
    let port = server_port;

    tokio::spawn(async move {
      Self::connection_loop(
        connection,
        profile,
        components,
        speedometer,
        plugins,
        reader_tx,
        writer_tx,
        storage,
        protocol_version,
        coonnection_timeout,
        proxy,
        host,
        port,
      )
      .await
    })
  }

  /// Метод запуска цикла подключения
  async fn connection_loop(
    connection: Arc<RwLock<Option<NurtexConnection>>>,
    profile: Arc<RwLock<BotProfile>>,
    components: Arc<RwLock<BotComponents>>,
    speedometer: Option<Arc<Speedometer>>,
    plugins: BotPlugins,
    reader_tx: PacketReader,
    writer_tx: PacketWriter,
    storage: Arc<RwLock<Storage>>,
    protocol_version: i32,
    coonnection_timeout: u64,
    proxy: Arc<RwLock<Option<Proxy>>>,
    host: String,
    port: u16,
  ) -> std::io::Result<()> {
    let mut reconnection_attempts = 0;
    let max_attempts = if plugins.auto_reconnect.enabled { plugins.auto_reconnect.max_attempts } else { 1 };

    loop {
      let reader_handle = Self::run_reader(Arc::clone(&connection), Arc::clone(&reader_tx));
      let writer_handle = Self::run_writer(Arc::clone(&connection), Arc::clone(&writer_tx));

      let result = Self::spawn_connection(
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
  }

  /// Метод спавна одного процесса подключения
  async fn spawn_connection(
    connection: &Arc<RwLock<Option<NurtexConnection>>>,
    profile: &Arc<RwLock<BotProfile>>,
    components: &Arc<RwLock<BotComponents>>,
    speedometer: &Option<Arc<Speedometer>>,
    plugins: &BotPlugins,
    reader_tx: &PacketReader,
    storage: &Arc<RwLock<Storage>>,
    protocol_version: i32,
    coonnection_timeout: u64,
    proxy: &Arc<RwLock<Option<Proxy>>>,
    host: &str,
    port: u16,
  ) -> std::io::Result<()> {
    {
      let mut conn_guard = connection.write().await;
      if let Some(conn) = conn_guard.as_ref() {
        let _ = conn.shutdown().await;
      }

      *conn_guard = None;
    }

    let conn = match proxy.read().await.as_ref() {
      Some(proxy) => match tokio::time::timeout(Duration::from_millis(coonnection_timeout), NurtexConnection::new_with_proxy(host, port, proxy)).await {
        Ok(result) => match result {
          Ok(c) => c,
          Err(err) => return Err(err),
        },
        Err(_) => return Err(Error::new(ErrorKind::TimedOut, "Failed to receive a response from the server within the specified timeout")),
      },
      None => match tokio::time::timeout(Duration::from_millis(coonnection_timeout), NurtexConnection::new(host, port)).await {
        Ok(result) => match result {
          Ok(c) => c,
          Err(err) => return Err(err),
        },
        Err(_) => return Err(Error::new(ErrorKind::TimedOut, "Failed to receive a response from the server within the specified timeout")),
      },
    };

    *connection.write().await = Some(conn);

    let profile_data = { profile.read().await.clone() };
    let username_for_speedometer = profile_data.username.clone();

    capture_connection(&connection, async |conn| {
      conn
        .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
          protocol_version: protocol_version,
          server_host: host.to_string(),
          server_port: port,
          intention: ClientIntention::Login,
        }))
        .await?;

      conn.set_state(ConnectionState::Login).await;

      conn
        .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart {
          username: profile_data.username.clone(),
          uuid: profile_data.uuid,
        }))
        .await
    })
    .await?;

    loop {
      let Some(packet) = ({
        let conn_guard = connection.read().await;
        if let Some(conn) = conn_guard.as_ref() { conn.read_login_packet().await } else { None }
      }) else {
        continue;
      };

      match packet {
        ClientsideLoginPacket::Compression(p) => {
          capture_connection(&connection, async |conn| {
            conn.set_compression_threshold(p.compression_threshold).await;
            Ok(())
          })
          .await?;
        }
        ClientsideLoginPacket::EncryptionRequest(request) => {
          if let Some((response, secret_key)) = handle_encryption_request(&request) {
            capture_connection(&connection, async |conn| {
              conn.write_login_packet(ServersideLoginPacket::EncryptionResponse(response)).await?;
              conn.set_encryption_key(secret_key).await;
              Ok(())
            })
            .await?;
          }
        }
        ClientsideLoginPacket::LoginSuccess(p) => {
          profile.write().await.uuid = p.uuid;
          capture_connection(&connection, async |conn| {
            conn.write_login_packet(ServersideLoginPacket::LoginAcknowledged(ServersideLoginAcknowledged)).await
          })
          .await?;
          break;
        }
        ClientsideLoginPacket::Disconnect(_p) => {
          return Err(Error::new(ErrorKind::ConnectionReset, "The connection was reset by the server"));
        }
        _ => {}
      }
    }

    capture_connection(&connection, async |conn| {
      conn.set_state(ConnectionState::Configuration).await;
      conn
        .write_configuration_packet(ServersideConfigurationPacket::ClientInformation(profile.read().await.information.to_serverside_packet()))
        .await
    })
    .await?;

    loop {
      let Some(packet) = ({
        let conn_guard = connection.read().await;
        if let Some(conn) = conn_guard.as_ref() {
          conn.read_configuration_packet().await
        } else {
          None
        }
      }) else {
        continue;
      };

      match packet {
        ClientsideConfigurationPacket::KeepAlive(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_configuration_packet(ServersideConfigurationPacket::KeepAlive(nurtex_protocol::packets::configuration::MultisideKeepAlive {
                id: p.id,
              }))
              .await
          })
          .await?;
        }
        ClientsideConfigurationPacket::Ping(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_configuration_packet(ServersideConfigurationPacket::Pong(nurtex_protocol::packets::configuration::ServersidePong { id: p.id }))
              .await
          })
          .await?;
        }
        ClientsideConfigurationPacket::KnownPacks(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_configuration_packet(ServersideConfigurationPacket::KnownPacks(ServersideKnownPacks { known_packs: p.known_packs }))
              .await
          })
          .await?;
        }
        ClientsideConfigurationPacket::FinishConfiguration(_) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_configuration_packet(ServersideConfigurationPacket::AcknowledgeFinishConfiguration(ServersideAcknowledgeFinishConfiguration))
              .await
          })
          .await?;
          break;
        }
        ClientsideConfigurationPacket::AddResourcePack(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_configuration_packet(ServersideConfigurationPacket::ResourcePackResponse(
                nurtex_protocol::packets::configuration::ServersideResourcePackResponse {
                  uuid: p.uuid,
                  state: ResourcePackState::Accepted,
                },
              ))
              .await
          })
          .await?;
        }
        ClientsideConfigurationPacket::Disconnect(_p) => {
          return Err(Error::new(ErrorKind::ConnectionReset, "The connection was reset by the server"));
        }
        _ => {}
      }
    }

    capture_connection(&connection, async |conn| {
      conn.set_state(ConnectionState::Play).await;
      Ok(())
    })
    .await?;

    if let Some(speedometer) = speedometer {
      speedometer.bot_joined(username_for_speedometer);
    }

    let mut packet_rx = {
      let reader_tx = Arc::clone(&reader_tx);
      reader_tx.subscribe()
    };

    loop {
      let packet = match tokio::time::timeout(Duration::from_millis(8000), packet_rx.recv()).await {
        Ok(Ok(ClientsidePacket::Play(play_packet))) => play_packet,
        Ok(Ok(_)) => continue,
        Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
        Ok(Err(broadcast::error::RecvError::Closed)) => return Err(Error::new(ErrorKind::ConnectionReset, "The connection was reset by the server")),
        Err(_) => continue,
      };

      match packet {
        ClientsidePlayPacket::Login(p) => {
          capture_components(&components, async |comp| {
            comp.entity_id = p.entity_id;
          })
          .await;

          if plugins.auto_respawn.enabled && p.enable_respawn_screen {
            tokio::time::sleep(Duration::from_millis(plugins.auto_respawn.respawn_delay)).await;

            capture_connection(&connection, async |conn| {
              conn
                .write_play_packet(ServersidePlayPacket::ClientCommand(ServersideClientCommand {
                  command: ClientCommand::PerformRespawn,
                }))
                .await
            })
            .await?;
          }
        }
        ClientsidePlayPacket::SpawnEntity(p) => {
          capture_storage(&storage, async |stor| {
            stor.add_entity(
              p.entity_id,
              Entity {
                entity_type: p.entity_type,
                entity_uuid: p.entity_uuid,
                position: p.position,
                rotation: Rotation::from_angle(p.yaw_angle, p.pitch_angle),
                velocity: p.velocity.to_vector3(),
                ..Default::default()
              },
            );
          })
          .await;
        }
        ClientsidePlayPacket::RemoveEntities(p) => {
          capture_storage(&storage, async |stor| {
            for entity_id in p.entities {
              stor.remove_entity(&entity_id);
            }
          })
          .await;
        }
        ClientsidePlayPacket::EntityPositionSync(p) => {
          capture_storage(&storage, async |stor| {
            if let Some(entity) = stor.get_entity_mut(&p.entity_id) {
              entity.position = p.position;
              entity.rotation = p.rotation;
              entity.velocity = p.velocity;
              entity.on_ground = p.on_ground;
            }
          })
          .await;
        }
        ClientsidePlayPacket::UpdateEntityPos(p) => {
          capture_storage(&storage, async |stor| {
            if let Some(entity) = stor.get_entity_mut(&p.entity_id) {
              // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
              entity.on_ground = p.on_ground;
            }
          })
          .await;
        }
        ClientsidePlayPacket::UpdateEntityRot(p) => {
          capture_storage(&storage, async |stor| {
            if let Some(entity) = stor.get_entity_mut(&p.entity_id) {
              entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
              entity.on_ground = p.on_ground;
            }
          })
          .await;
        }
        ClientsidePlayPacket::UpdateEntityPosRot(p) => {
          capture_storage(&storage, async |stor| {
            if let Some(entity) = stor.get_entity_mut(&p.entity_id) {
              // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
              entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
              entity.on_ground = p.on_ground;
            }
          })
          .await;
        }
        ClientsidePlayPacket::SetEntityVelocity(p) => {
          capture_storage(&storage, async |stor| {
            if let Some(entity) = stor.get_entity_mut(&p.entity_id) {
              entity.position.with_velocity(p.velocity.to_vector3());
              entity.velocity = Vector3::from_lp_vector3(p.velocity);
            }
          })
          .await;
        }
        ClientsidePlayPacket::KeepAlive(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::KeepAlive(nurtex_protocol::packets::play::MultisideKeepAlive { id: p.id }))
              .await
          })
          .await?;
        }
        ClientsidePlayPacket::Ping(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::Pong(nurtex_protocol::packets::play::ServersidePong { id: p.id }))
              .await
          })
          .await?;
        }
        ClientsidePlayPacket::SetHealth(p) => {
          capture_components(&components, async |comp| {
            comp.health = p.health;
            comp.food = p.food;
          })
          .await;
        }
        ClientsidePlayPacket::SetExperience(p) => {
          capture_components(&components, async |comp| {
            comp.experience = p.experience;
          })
          .await;
        }
        ClientsidePlayPacket::PlayerPosition(p) => {
          capture_components(&components, async |comp| {
            comp.position = p.position;
            comp.velocity = p.velocity;
            comp.rotation = p.rotation;
          })
          .await;

          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::AcceptTeleportation(ServersideAcceptTeleportation { teleport_id: p.teleport_id }))
              .await
          })
          .await?;
        }
        ClientsidePlayPacket::PlayerRotation(p) => {
          capture_components(&components, async |comp| {
            comp.rotation = Rotation::new(p.yaw, p.pitch);
          })
          .await;
        }
        ClientsidePlayPacket::AddResourcePack(p) => {
          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::ResourcePackResponse(nurtex_protocol::packets::play::ServersideResourcePackResponse {
                uuid: p.uuid,
                state: ResourcePackState::Accepted,
              }))
              .await
          })
          .await?;
        }
        ClientsidePlayPacket::PlayerCombatKill(_p) => {
          if plugins.auto_respawn.enabled {
            tokio::time::sleep(Duration::from_millis(plugins.auto_respawn.respawn_delay)).await;

            capture_connection(&connection, async |conn| {
              conn
                .write_play_packet(ServersidePlayPacket::ClientCommand(ServersideClientCommand {
                  command: ClientCommand::PerformRespawn,
                }))
                .await
            })
            .await?;
          }
        }
        ClientsidePlayPacket::Disconnect(_p) => {
          return Err(Error::new(ErrorKind::ConnectionReset, "The connection was reset by the server"));
        }
        _ => {}
      }
    }
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
    capture_components(&self.components, async |comp| {
      *comp = BotComponents::default();
    })
    .await;
  }

  /// Метод отмены хэндла бота
  pub fn abort_handle(&self) {
    if let Some(handle) = &self.handle {
      handle.abort();
    }
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
    match self.storage.try_read() {
      Ok(g) => {
        let mut entities = HashMap::with_capacity(g.entities.len());

        for (entity_id, entity) in &g.entities {
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
    let guard = self.storage.read().await;
    let mut entities = HashMap::with_capacity(guard.entities.len());

    for (entity_id, entity) in &guard.entities {
      entities.insert(*entity_id, entity.clone());
    }

    entities
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use nurtex_protocol::connection::ClientsidePacket;
  use nurtex_protocol::packets::play::ClientsidePlayPacket;
  use nurtex_proxy::Proxy;

  use crate::bot::plugins::{AutoReconnectPlugin, AutoRespawnPlugin, BotPlugins};
  use crate::bot::{Bot, BotChatExt};

  #[tokio::test]
  async fn test_packet_handling() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").set_protocol_version(773);

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
  async fn test_auto_respawn() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot")
      .set_plugins(BotPlugins {
        auto_respawn: AutoRespawnPlugin {
          enabled: true,
          respawn_delay: 2000,
        },
        ..Default::default()
      })
      .set_protocol_version(773);

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }

  #[tokio::test]
  async fn test_auto_reconnect() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot")
      .set_plugins(BotPlugins {
        auto_reconnect: AutoReconnectPlugin {
          enabled: true,
          reconnect_delay: 1000,
          max_attempts: 3,
        },
        ..Default::default()
      })
      .set_protocol_version(773);

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
  async fn test_storage() -> io::Result<()> {
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
  async fn test_bot_with_proxy() -> io::Result<()> {
    let proxy = Proxy::new("212.58.132.5:1080"); // Публичный прокси
    let mut bot = Bot::create_with_proxy("l7jqw8d5", proxy);

    bot.connect("hub.holyworld.ru", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }
}
