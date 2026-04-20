use std::io::{self, Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use nurtex_protocol::connection::NurtexConnection;
use nurtex_protocol::connection::address::convert_address;
use nurtex_protocol::connection::utils::handle_encryption_request;
use nurtex_protocol::connection::{ClientsidePacket, ConnectionState};
use nurtex_protocol::packets::configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks};
use nurtex_protocol::packets::configuration::{ServersideResourcePackResponse};
use nurtex_protocol::packets::handshake::{ServersideGreet, ServersideHandshakePacket};
use nurtex_protocol::packets::login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart};
use nurtex_protocol::packets::play::ServersidePlayPacket;
use nurtex_protocol::types::{ClientIntention, ResourcePackState};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::profile::ClientProfile;
use crate::structs::ClientInfo;
use crate::version::get_protocol_from_version;

/// Структура Minecraft клиента
pub struct Client {
  profile: Arc<RwLock<ClientProfile>>,
  handle: Option<JoinHandle<Result<(), Error>>>,
  connection: Arc<RwLock<Option<NurtexConnection>>>,
  reader: Arc<broadcast::Sender<ClientsidePacket>>,
  writer: Arc<broadcast::Sender<ServersidePlayPacket>>,
}

impl Client {
  /// Метод создания нового клиента
  pub fn create(username: impl Into<String>, version: impl Into<String>) -> Self {
    let (reader_tx, _) = broadcast::channel(45);
    let (writer_tx, _) = broadcast::channel(45);

    let reader = Arc::new(reader_tx);
    let writer = Arc::new(writer_tx);
    let conn = Arc::new(RwLock::new(None::<NurtexConnection>));

    Self::run_reader(conn.clone(), reader.clone());
    Self::run_writer(conn.clone(), writer.clone());

    let profile = ClientProfile::new(username.into(), get_protocol_from_version(&version.into()));

    Self {
      profile: Arc::new(RwLock::new(profile)),
      handle: None,
      connection: conn,
      reader: reader,
      writer: writer,
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при создании клиента через `Client::create`)
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
          sleep(Duration::from_millis(10)).await;
        }
      }
    });
  }

  /// Метод запуска `writer` (выполняется автоматически при создании клиента через `Client::create`)
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
  pub async fn get_username(&self) -> String {
    self.profile.read().await.username.clone()
  }

  /// Метод получения профиля клиента
  pub fn get_profile(&self) -> Arc<RwLock<ClientProfile>> {
    self.profile.clone()
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

  /// Метод отправки пакета в `broadcast` канал
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer.send(packet);
  }

  /// Метод создания соединения с сервером и отправки базовой информации
  pub fn connect_to(&mut self, server_host: impl Into<String>, server_port: u16) {
    let connection = self.connection.clone();
    let profile = self.profile.clone();

    let host = server_host.into();

    let handle = tokio::spawn(async move {
      let Some(addr): Option<std::net::SocketAddr> = convert_address(format!("{}:{}", host, server_port)) else {
        return Err(Error::new(ErrorKind::AddrNotAvailable, "Failed to convert target address"));
      };

      let mut conn = match NurtexConnection::new(&addr).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
      };

      let (username, uuid, protocol_version) = {
        let guard = profile.read().await;
        (guard.username.clone(), guard.uuid, guard.protocol_version)
      };

      conn
        .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
          protocol_version: protocol_version,
          server_host: addr.ip().to_string(),
          server_port: addr.port(),
          intention: ClientIntention::Login,
        }))
        .await?;

      conn.set_state(ConnectionState::Login);

      conn
        .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart { username: username, uuid: uuid }))
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

      Ok(())
    });

    self.handle = Some(handle);
  }

  /// Метод отключения клиента
  pub async fn shutdown(&self) -> io::Result<()> {
    if let Some(handle) = &self.handle {
      handle.abort();
    } else if let Some(conn) = self.connection.write().await.as_mut() {
      conn.shutdown().await?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use nurtex_protocol::connection::ClientsidePacket;
  use nurtex_protocol::packets::play::{ClientsidePlayPacket, ServersidePlayPacket};

  use crate::Client;

  #[tokio::test]
  async fn create_client() -> io::Result<()> {
    for i in 0..6 {
      tokio::spawn(async move {
        let mut client = Client::create(format!("NurtexBot_{}", i), "1.21.11");

        client.connect_to("localhost", 25565);

        let reader = client.get_reader();
        let mut packet_rx = reader.subscribe();

        loop {
          if let Ok(ClientsidePacket::Play(packet)) = packet_rx.recv().await {
            println!("Получен пакет: {:?}", packet);

            match packet {
              ClientsidePlayPacket::KeepAlive(p) => {
                client.send_packet(ServersidePlayPacket::KeepAlive(nurtex_protocol::packets::play::MultisideKeepAlive { id: p.id }));
              }
              _ => {}
            }
          }
        }
      });
    }

    tokio::time::sleep(Duration::from_secs(30)).await;

    Ok(())
  }
}
