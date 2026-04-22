use std::io::{self, Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use nurtex_protocol::connection::address::convert_address;
use nurtex_protocol::connection::utils::handle_encryption_request;
use nurtex_protocol::connection::{ClientsidePacket, ConnectionState, NurtexConnection};
use nurtex_protocol::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks, ServersideResourcePackResponse},
  handshake::{ServersideGreet, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart},
  play::ServersidePlayPacket,
};
use nurtex_protocol::types::{ClientIntention, ResourcePackState};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

use crate::bot::{BotProfile, ClientInfo};
use crate::swarm::{Speedometer, SwarmObject};

/// Структура Minecraft бота
pub struct Bot {
  username: String,
  profile: Arc<RwLock<BotProfile>>,
  handle: Option<JoinHandle<Result<(), Error>>>,
  connection: Arc<RwLock<Option<NurtexConnection>>>,
  reader: Arc<broadcast::Sender<ClientsidePacket>>,
  writer: Arc<broadcast::Sender<ServersidePlayPacket>>,
  speedometer: Option<Arc<Speedometer>>,
}

impl Bot {
  /// Метод создания нового бота
  pub fn create(username: impl Into<String>) -> Self {
    Self::create_with_options(username, 45, 45, None)
  }

  /// Метод создания нового бота из объекта роя
  pub fn create_from_object(object: SwarmObject) -> Self {
    Self::create_with_options(object.username, object.reader_capacity, object.writer_capacity, None)
  }

  /// Метод создания нового бота со спидометром
  pub fn create_with_speedometer(username: impl Into<String>, speedometer: Arc<Speedometer>) -> Self {
    Self::create_with_options(username, 45, 45, Some(speedometer))
  }

  /// Метод создания нового бота из объекта роя со спидометром
  pub fn create_from_object_with_speedometer(object: SwarmObject, speedometer: Arc<Speedometer>) -> Self {
    Self::create_with_options(object.username, object.reader_capacity, object.writer_capacity, Some(speedometer))
  }

  /// Метод создания нового бота с заданными опциями
  pub fn create_with_options(
    username: impl Into<String>,
    reader_capacity: usize,
    writer_capacity: usize,
    speedometer: Option<Arc<Speedometer>>,
  ) -> Self {
    let (reader_tx, _) = broadcast::channel(reader_capacity);
    let (writer_tx, _) = broadcast::channel(writer_capacity);

    let reader = Arc::new(reader_tx);
    let writer = Arc::new(writer_tx);
    let conn = Arc::new(RwLock::new(None::<NurtexConnection>));

    Self::run_reader(conn.clone(), reader.clone());
    Self::run_writer(conn.clone(), writer.clone());

    let name = username.into();
    let profile = BotProfile::new(name.clone());

    Self {
      username: name,
      profile: Arc::new(RwLock::new(profile)),
      handle: None,
      connection: conn,
      reader: reader,
      writer: writer,
      speedometer,
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при создании бота через `Bot::create`)
  pub fn run_reader(connection: Arc<RwLock<Option<NurtexConnection>>>, reader_tx: Arc<broadcast::Sender<ClientsidePacket>>) {
    tokio::spawn(async move {
      loop {
        let packet = {
          let mut guard = connection.write().await;
          if let Some(conn) = guard.as_mut() { conn.read_packet().await } else { None }
        };

        if let Some(packet) = packet {
          let _ = reader_tx.send(packet);
        } else {
          tokio::time::sleep(Duration::from_millis(10)).await;
        }
      }
    });
  }

  /// Метод запуска `writer` (выполняется автоматически при создании бота через `Bot::create`)
  pub fn run_writer(connection: Arc<RwLock<Option<NurtexConnection>>>, writer_tx: Arc<broadcast::Sender<ServersidePlayPacket>>) {
    let mut rx = writer_tx.subscribe();

    tokio::spawn(async move {
      loop {
        if let Ok(packet) = rx.recv().await {
          let mut conn_guard = connection.write().await;
          if let Some(conn) = conn_guard.as_mut() {
            let _ = conn.write_play_packet(packet).await;
          }
        }
      }
    });
  }

  /// Метод установки информации клиента
  pub async fn set_information(&self, information: ClientInfo) {
    self.profile.write().await.information = information;
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
    self.reader.subscribe()
  }

  /// Метод получения копии `reader`
  pub fn get_reader(&self) -> Arc<broadcast::Sender<ClientsidePacket>> {
    Arc::clone(&self.reader)
  }

  /// Метод получения копии `writer`
  pub fn get_writer(&self) -> Arc<broadcast::Sender<ServersidePlayPacket>> {
    Arc::clone(&self.writer)
  }

  /// Метод получения копии подключения
  pub fn get_connection(&self) -> Arc<RwLock<Option<NurtexConnection>>> {
    Arc::clone(&self.connection)
  }

  /// Метод получения хэндла
  pub fn get_handle(&self) -> &Option<JoinHandle<Result<(), Error>>> {
    &self.handle
  }

  /// Метод отправки пакета
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer.send(packet);
  }

  /// Метод подключения бота к серверу
  pub fn connect(&mut self, server_host: impl Into<String>, server_port: u16) {
    let handle = self.connect_with_handle(server_host, server_port);
    self.handle = Some(handle);
  }

  /// Метод подключения бота к серверу, который возвращает `handle` задачи подключения
  pub fn connect_with_handle(&self, server_host: impl Into<String>, server_port: u16) -> JoinHandle<Result<(), Error>> {
    let connection = self.connection.clone();
    let profile = self.profile.clone();
    let speedometer = self.speedometer.clone();
    let host = server_host.into();

    tokio::spawn(async move {
      let Some(addr) = convert_address(format!("{}:{}", host, server_port)) else {
        return Err(Error::new(ErrorKind::AddrNotAvailable, "Failed to convert target address"));
      };

      let mut conn = match NurtexConnection::new(&addr).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
      };

      let profile_data = { profile.read().await.clone() };

      conn
        .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
          protocol_version: 774,
          server_host: host,
          server_port: server_port,
          intention: ClientIntention::Login,
        }))
        .await?;

      conn.set_state(ConnectionState::Login);

      conn
        .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart {
          username: profile_data.username.clone(),
          uuid: profile_data.uuid,
        }))
        .await?;

      loop {
        if let Some(p) = conn.read_login_packet().await {
          match p {
            ClientsideLoginPacket::Compression(p) => {
              conn.set_compression_threshold(p.compression_threshold);
            }
            ClientsideLoginPacket::EncryptionRequest(request) => {
              if let Some((response, secret_key)) = handle_encryption_request(&request) {
                conn.write_login_packet(ServersideLoginPacket::EncryptionResponse(response)).await?;
                conn.set_encryption_key(secret_key);
              }
            }
            ClientsideLoginPacket::LoginSuccess(p) => {
              profile.write().await.uuid = p.uuid;
              conn.write_login_packet(ServersideLoginPacket::LoginAcknowledged(ServersideLoginAcknowledged)).await?;
              break;
            }
            _ => {}
          }
        } else {
          break;
        }
      }

      conn.set_state(ConnectionState::Configuration);

      conn
        .write_configuration_packet(ServersideConfigurationPacket::ClientInformation(profile.read().await.information.to_serverside_packet()))
        .await?;

      loop {
        if let Some(p) = conn.read_configuration_packet().await {
          match p {
            ClientsideConfigurationPacket::KeepAlive(p) => {
              conn
                .write_configuration_packet(ServersideConfigurationPacket::KeepAlive(nurtex_protocol::packets::configuration::MultisideKeepAlive {
                  id: p.id,
                }))
                .await?;
            }
            ClientsideConfigurationPacket::Ping(p) => {
              conn
                .write_configuration_packet(ServersideConfigurationPacket::Pong(nurtex_protocol::packets::configuration::ServersidePong { id: p.id }))
                .await?;
            }
            ClientsideConfigurationPacket::KnownPacks(p) => {
              conn
                .write_configuration_packet(ServersideConfigurationPacket::KnownPacks(ServersideKnownPacks { known_packs: p.known_packs }))
                .await?;
            }
            ClientsideConfigurationPacket::FinishConfiguration(_) => {
              conn
                .write_configuration_packet(ServersideConfigurationPacket::AcknowledgeFinishConfiguration(ServersideAcknowledgeFinishConfiguration))
                .await?;
              break;
            }
            ClientsideConfigurationPacket::AddResourcePack(p) => {
              conn
                .write_configuration_packet(ServersideConfigurationPacket::ResourcePackResponse(ServersideResourcePackResponse {
                  uuid: p.uuid,
                  state: ResourcePackState::Accepted,
                }))
                .await?;
            }
            _ => {}
          }
        } else {
          break;
        }
      }

      conn.set_state(ConnectionState::Play);

      *connection.write().await = Some(conn);

      if let Some(speedometer) = speedometer {
        speedometer.bot_joined(profile_data.username);
      }

      Ok(())
    })
  }

  /// Метод отключения клиента
  pub async fn shutdown(&self) -> io::Result<()> {
    if let Some(handle) = &self.handle {
      handle.abort();
      *self.connection.write().await = None;
    } else if let Some(conn) = self.connection.write().await.as_mut() {
      conn.shutdown().await?;
    }

    Ok(())
  }

  /// Метод отмены хэндла клиента
  pub fn abort_handle(&self) {
    if let Some(handle) = &self.handle {
      handle.abort();
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{io, time::Duration};

  use nurtex_protocol::{
    connection::ClientsidePacket,
    packets::play::{ClientsidePlayPacket, MultisideKeepAlive, ServersidePlayPacket, ServersideSwingArm},
    types::RelativeHand,
  };

  use crate::bot::Bot;

  #[tokio::test]
  async fn test_packet_handling() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);

        match packet {
          ClientsidePlayPacket::KeepAlive(p) => {
            bot.send_packet(ServersidePlayPacket::KeepAlive(MultisideKeepAlive { id: p.id }));
            bot.send_packet(ServersidePlayPacket::SwingArm(ServersideSwingArm { hand: RelativeHand::MainHand }));
            tokio::time::sleep(Duration::from_secs(2)).await;
            break;
          }
          _ => {}
        }
      }
    }

    Ok(())
  }
}
