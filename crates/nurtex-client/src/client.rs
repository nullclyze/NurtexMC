use std::io::{self, Error, ErrorKind};
use std::sync::Arc;

use nurtex_protocol::connection::NurtexConnection;
use nurtex_protocol::connection::address::convert_address;
use nurtex_protocol::connection::utils::handle_encryption_request;
use nurtex_protocol::connection::{ClientsidePacket, ConnectionState};
use nurtex_protocol::packets::configuration::{ResourcePackState, ServersideResourcePackResponse};
use nurtex_protocol::packets::play::ServersidePlayPacket;
use nurtex_protocol::packets::configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks};
use nurtex_protocol::packets::handshake::{ClientIntention, ServersideGreet, ServersideHandshakePacket};
use nurtex_protocol::packets::login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::structs::ClientInfo;
use crate::version::get_protocol_from_version;

/// Структура Minecraft клиента
pub struct Client {
  username: String,
  uuid: Uuid,
  information: ClientInfo,
  protocol_version: i32,
  connection: Arc<RwLock<Option<NurtexConnection>>>,
  reader_tx: Arc<broadcast::Sender<ClientsidePacket>>,
  writer_tx: Arc<broadcast::Sender<ServersidePlayPacket>>,
}

impl Client {
  /// Метод создания нового клиента
  pub fn create(username: impl Into<String>, version: impl Into<String>) -> Self {
    let (reader_tx, _) = broadcast::channel(50);
    let (writer_tx, mut writer_rx) = broadcast::channel(50);

    let reader = Arc::new(reader_tx);
    let writer = Arc::new(writer_tx);

    let conn = Arc::new(RwLock::new(None::<NurtexConnection>));

    let writer_conn = conn.clone();

    tokio::spawn(async move {
      loop {
        match writer_rx.recv().await {
          Ok(packet) => {
            let mut conn_guard = writer_conn.write().await;
            if let Some(conn) = conn_guard.as_mut() {
              let _ = conn.write_play_packet(packet).await;
            }
          }
          Err(broadcast::error::RecvError::Lagged(_)) => continue,
          Err(broadcast::error::RecvError::Closed) => break,
        }
      }
    });

    let reader_conn = conn.clone();
    let reader_clone = reader.clone();

    tokio::spawn(async move {
      loop {
        let packet = {
          let mut guard = reader_conn.write().await;
          if let Some(conn) = guard.as_mut() { conn.read_packet().await } else { None }
        };

        if let Some(packet) = packet {
          let _ = reader_clone.send(packet);
        } else {
          tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
      }
    });

    Self {
      username: username.into(),
      uuid: Uuid::nil(),
      information: ClientInfo::default(),
      protocol_version: get_protocol_from_version(&version.into()),
      connection: conn,
      reader_tx: reader,
      writer_tx: writer,
    }
  }

  /// Метод установки UUID
  pub fn set_uuid(&mut self, uuid: Uuid) {
    self.uuid = uuid;
  }

  /// Метод установки информации клиента
  pub fn set_information(mut self, information: ClientInfo) -> Self {
    self.information = information;
    self
  }

  /// Метод получения ссылочного юзернейма
  pub fn see_username(&self) -> &String {
    &self.username
  }

  /// Метод получения юзернейма
  pub fn get_username(&self) -> String {
    self.username.clone()
  }

  /// Метод подписки на слушание пакетов
  pub fn subscribe_to_reader(&self) -> broadcast::Receiver<ClientsidePacket> {
    self.reader_tx.subscribe()
  }

  /// Метод получения копии `writer`
  pub fn get_writer_tx(&self) -> Arc<broadcast::Sender<ServersidePlayPacket>> {
    self.writer_tx.clone()
  }

  /// Метод отправки пакета в `broadcast` канал
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer_tx.send(packet);
  }

  /// Метод создания соединения с сервером и отправки базовой информации
  pub async fn connect_to(&mut self, server_host: impl Into<String>, server_port: u16) -> io::Result<()> {
    let Some(addr): Option<std::net::SocketAddr> = convert_address(format!("{}:{}", server_host.into(), server_port)) else {
      return Err(Error::new(ErrorKind::AddrNotAvailable, "Failed to convert target address"));
    };

    let mut conn = match NurtexConnection::new(&addr).await {
      Ok(c) => c,
      Err(_) => return Ok(()),
    };

    conn
      .write_handshake_packet(ServersideHandshakePacket::Intention(ServersideGreet {
        protocol_version: self.protocol_version,
        server_host: addr.ip().to_string(),
        server_port: addr.port(),
        intention: ClientIntention::Login,
      }))
      .await?;

    conn.set_state(ConnectionState::Login);

    conn
      .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart {
        username: self.get_username(),
        uuid: self.uuid,
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
            self.set_uuid(p.uuid);
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
      .write_configuration_packet(ServersideConfigurationPacket::ClientInformation(self.information.to_serverside_packet()))
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

    *self.connection.write().await = Some(conn);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{io, time::Duration};

  use nurtex_protocol::{
    connection::ClientsidePacket,
    packets::play::{ClientsidePlayPacket, ServersidePlayPacket},
  };

  use crate::Client;

  #[tokio::test]
  async fn create_client() -> io::Result<()> {
    for i in 0..6 {
      tokio::spawn(async move {
        let mut client = Client::create(format!("NurtexBot_{}", i), "1.21.11");

        let _ = client.connect_to("localhost", 25565).await;

        let mut packet_rx = client.subscribe_to_reader();

        loop {
          if let Ok(p) = packet_rx.recv().await {
            println!("Получен пакет: {:?}", p);
            match p {
              ClientsidePacket::Play(ClientsidePlayPacket::KeepAlive(p)) => {
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
