use std::io::{self, Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use nurtex_protocol::connection::address::convert_address;
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
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

use crate::bot::capture::{capture_components, capture_connection};
use crate::bot::plugins::BotPlugins;
use crate::bot::{BotComponents, BotHandle, BotProfile, ClientInfo};
use crate::swarm::Speedometer;

/// Структура Minecraft бота
pub struct Bot {
  pub profile: Arc<RwLock<BotProfile>>,
  pub connection: Arc<RwLock<Option<NurtexConnection>>>,
  plugins: BotPlugins,
  username: String,
  handle: BotHandle,
  reader_tx: Arc<broadcast::Sender<ClientsidePacket>>,
  writer_tx: Arc<broadcast::Sender<ServersidePlayPacket>>,
  speedometer: Option<Arc<Speedometer>>,
  components: Arc<RwLock<BotComponents>>,
}

impl Bot {
  /// Метод создания нового бота
  pub fn create(username: impl Into<String>) -> Self {
    Self::create_with_options(username, 45, 45, None)
  }

  /// Метод создания нового бота со спидометром
  pub fn create_with_speedometer(username: impl Into<String>, speedometer: Arc<Speedometer>) -> Self {
    Self::create_with_options(username, 45, 45, Some(speedometer))
  }

  /// Метод создания нового бота с заданными опциями
  pub fn create_with_options(username: impl Into<String>, reader_capacity: usize, writer_capacity: usize, speedometer: Option<Arc<Speedometer>>) -> Self {
    let (reader_tx, _) = broadcast::channel(reader_capacity);
    let (writer_tx, _) = broadcast::channel(writer_capacity);

    let name = username.into();
    let profile = BotProfile::new(name.clone());

    Self {
      profile: Arc::new(RwLock::new(profile)),
      connection: Arc::new(RwLock::new(None::<NurtexConnection>)),
      plugins: BotPlugins::default(),
      username: name,
      handle: BotHandle::default(),
      reader_tx: Arc::new(reader_tx),
      writer_tx: Arc::new(writer_tx),
      speedometer,
      components: Arc::new(RwLock::new(BotComponents::default())),
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при подключении бота)
  pub fn run_reader(&self) -> JoinHandle<()> {
    let reader_tx = Arc::clone(&self.reader_tx);
    let connection = Arc::clone(&self.connection);

    tokio::spawn(async move {
      // Может быть гонка условий с NurtexConnection, поэтому небольшая задержка нужна
      tokio::time::sleep(Duration::from_millis(600)).await;

      loop {
        let has_connection = {
          let conn_guard = connection.read().await;
          conn_guard.is_some()
        };

        if !has_connection {
          tokio::time::sleep(Duration::from_millis(100)).await;
          continue;
        }

        let packet = {
          let conn_guard = connection.read().await;
          if let Some(conn) = conn_guard.as_ref() { conn.read_packet().await } else { None }
        };

        if let Some(packet) = packet {
          let _ = reader_tx.send(packet);
        } else {
          tokio::time::sleep(Duration::from_millis(10)).await;
        }
      }
    })
  }

  /// Метод запуска `writer` (выполняется автоматически при подключении бота)
  pub fn run_writer(&self) -> JoinHandle<()> {
    let writer_tx = Arc::clone(&self.writer_tx);
    let mut writer_rx = writer_tx.subscribe();
    let connection = Arc::clone(&self.connection);

    tokio::spawn(async move {
      // Может быть гонка условий с NurtexConnection, поэтому небольшая задержка нужна
      tokio::time::sleep(Duration::from_millis(1000)).await;

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

  /// Метод получения юзернейма
  pub fn username(&self) -> &str {
    &self.username
  }

  /// Метод получения профиля бота
  pub fn get_profile(&self) -> Arc<RwLock<BotProfile>> {
    Arc::clone(&self.profile)
  }

  /// Вспомогательный метод подписки на слушание пакетов
  pub fn subscribe_to_reader(&self) -> broadcast::Receiver<ClientsidePacket> {
    self.reader_tx.subscribe()
  }

  /// Метод получения копии `reader_tx`
  pub fn get_reader(&self) -> Arc<broadcast::Sender<ClientsidePacket>> {
    Arc::clone(&self.reader_tx)
  }

  /// Метод получения копии `writer_tx`
  pub fn get_writer(&self) -> Arc<broadcast::Sender<ServersidePlayPacket>> {
    Arc::clone(&self.writer_tx)
  }

  /// Метод получения копии подключения
  pub fn get_connection(&self) -> Arc<RwLock<Option<NurtexConnection>>> {
    Arc::clone(&self.connection)
  }

  /// Метод получения хэндла
  pub fn get_handle(&self) -> &BotHandle {
    &self.handle
  }

  /// Метод отправки пакета
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer_tx.send(packet);
  }

  /// Метод подключения бота к серверу
  pub fn connect(&mut self, server_host: impl Into<String>, server_port: u16) {
    self.handle = self.connect_with_handle(server_host, server_port);
  }

  /// Метод подключения бота к серверу, который возвращает хендл бота
  pub fn connect_with_handle(&self, server_host: impl Into<String>, server_port: u16) -> BotHandle {
    let connection = Arc::clone(&self.connection);
    let profile = Arc::clone(&self.profile);
    let components = Arc::clone(&self.components);
    let speedometer = self.speedometer.clone();
    let plugins = self.plugins.clone();
    let reader_tx = Arc::clone(&self.reader_tx);
    let host = server_host.into();

    let connection_handle = tokio::spawn(async move {
      let Some(addr) = convert_address(format!("{}:{}", host, server_port)) else {
        return Err(Error::new(ErrorKind::AddrNotAvailable, "Failed to convert target address"));
      };

      let conn = match NurtexConnection::new(&addr).await {
        Ok(c) => c,
        Err(err) => return Err(err),
      };

      *connection.write().await = Some(conn);

      let profile_data = { profile.read().await.clone() };
      let username_for_speedometer = profile_data.username.clone();

      capture_connection(&connection, async |conn| {
        conn
          .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
            protocol_version: 774,
            server_host: host.clone(),
            server_port: server_port,
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
        let packet = match packet_rx.recv().await {
          Ok(ClientsidePacket::Play(play_packet)) => play_packet,
          Ok(_) => continue,
          Err(broadcast::error::RecvError::Lagged(_)) => {
            continue;
          }
          Err(broadcast::error::RecvError::Closed) => {
            break Ok(());
          }
        };

        match packet {
          ClientsidePlayPacket::Login(p) => {
            capture_components(&components, async |comp| {
              comp.entity_id = p.entity_id;
              Ok(())
            })
            .await?;

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
              Ok(())
            })
            .await?;
          }
          ClientsidePlayPacket::SetExperience(p) => {
            capture_components(&components, async |comp| {
              comp.experience = p.experience;
              Ok(())
            })
            .await?;
          }
          ClientsidePlayPacket::PlayerPosition(p) => {
            capture_components(&components, async |comp| {
              comp.position = p.position;
              comp.velocity = p.velocity;
              comp.rotation = p.rotation;
              Ok(())
            })
            .await?;

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
              Ok(())
            })
            .await?;
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
            return Ok(());
          }
          _ => {}
        }
      }
    });

    BotHandle {
      connection_handle: Some(connection_handle),
      reader_handle: Some(self.run_reader()),
      writer_handle: Some(self.run_writer()),
    }
  }

  /// Метод ожидания завершения хэндла подключения
  pub async fn wait_handle(&mut self) -> std::io::Result<()> {
    if let Some(handle) = self.handle.connection_handle.as_mut() {
      handle.await?
    } else {
      Ok(())
    }
  }

  /// Метод полноценной очистки и отключения бота
  pub async fn shutdown(&self) -> io::Result<()> {
    self.abort_handle();

    let mut conn_guard = self.connection.write().await;
    if let Some(conn) = conn_guard.as_ref() {
      conn.shutdown().await?;
    }

    *conn_guard = None;
    std::mem::drop(conn_guard);

    self.clear().await
  }

  /// Метод очистки данных бота
  pub async fn clear(&self) -> io::Result<()> {
    capture_components(&self.components, async |comp| {
      *comp = BotComponents::default();
      Ok(())
    })
    .await
  }

  /// Метод отмены хэндла бота
  pub fn abort_handle(&self) {
    self.handle.abort();
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
}

#[cfg(test)]
mod tests {
  use std::io;

  use nurtex_protocol::connection::ClientsidePacket;

  use crate::bot::Bot;
  use crate::bot::plugins::{AutoRespawnPlugin, BotPlugins};

  #[tokio::test]
  async fn test_packet_handling() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_auto_respawn() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").set_plugins(BotPlugins {
      auto_respawn: AutoRespawnPlugin {
        enabled: true,
        respawn_delay: 2000,
      },
    });

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }
}
