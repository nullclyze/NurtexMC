use std::io::{self, Error, ErrorKind};

use azalea_protocol::common::client_information::ClientInformation;
use azalea_protocol::connect::Connection;
use azalea_protocol::packets::config::{ClientboundConfigPacket, ServerboundConfigPacket};
use azalea_protocol::packets::login::{ClientboundLoginPacket, ServerboundLoginPacket};

/// Функция обработки всего цикла пакетов в состоянии Login
pub async fn handle_login(
  conn: &mut Connection<ClientboundLoginPacket, ServerboundLoginPacket>,
) -> io::Result<()> {
  use azalea_protocol::packets::login::*;

  loop {
    let packet = match conn.read().await {
      Ok(p) => p,
      Err(_) => continue,
    };

    match packet {
      ClientboundLoginPacket::Hello(p) => {
        let e = azalea_crypto::encrypt(&p.public_key, &p.challenge).unwrap();

        conn
          .write(ServerboundKey {
            key_bytes: e.encrypted_public_key,
            encrypted_challenge: e.encrypted_challenge,
          })
          .await?;

        conn.set_encryption_key(e.secret_key);
      }
      ClientboundLoginPacket::LoginCompression(p) => {
        conn.set_compression_threshold(p.compression_threshold);
      }
      ClientboundLoginPacket::LoginFinished(_p) => {
        return Ok(());
      }
      ClientboundLoginPacket::LoginDisconnect(p) => {
        return Err(Error::new(
          ErrorKind::ConnectionAborted,
          format!("Bot was disconnected (login): {}", p.reason.to_string()),
        ));
      }
      ClientboundLoginPacket::CookieRequest(p) => {
        conn
          .write(ServerboundCookieResponse {
            key: p.key,
            payload: None,
          })
          .await?;
      }
      _ => {}
    }
  }
}

/// Функция обработки всего цикла пакетов в состоянии Configuration
pub async fn handle_configuration(
  conn: &mut Connection<ClientboundConfigPacket, ServerboundConfigPacket>,
  client_information: ClientInformation,
) -> io::Result<()> {
  use azalea_protocol::packets::config::*;

  conn
    .write(ServerboundConfigPacket::ClientInformation(
      s_client_information::ServerboundClientInformation {
        information: client_information,
      },
    ))
    .await?;

  loop {
    let packet = match conn.read().await {
      Ok(p) => p,
      Err(_) => continue,
    };

    match packet {
      ClientboundConfigPacket::RegistryData(_) => {}
      ClientboundConfigPacket::UpdateTags(_) => {}
      ClientboundConfigPacket::SelectKnownPacks(_) => {
        conn
          .write(ServerboundConfigPacket::SelectKnownPacks(
            s_select_known_packs::ServerboundSelectKnownPacks {
              known_packs: vec![],
            },
          ))
          .await?;
      }
      ClientboundConfigPacket::KeepAlive(p) => {
        conn
          .write(ServerboundConfigPacket::KeepAlive(
            s_keep_alive::ServerboundKeepAlive { id: p.id },
          ))
          .await?;
      }
      ClientboundConfigPacket::FinishConfiguration(_) => {
        conn
          .write(ServerboundConfigPacket::FinishConfiguration(
            s_finish_configuration::ServerboundFinishConfiguration {},
          ))
          .await?;
        return Ok(());
      }
      ClientboundConfigPacket::Disconnect(p) => {
        return Err(Error::new(
          ErrorKind::ConnectionAborted,
          format!("Bot was disconnected (config): {}", p.reason.to_string()),
        ));
      }
      ClientboundConfigPacket::CookieRequest(p) => {
        conn
          .write(ServerboundConfigPacket::CookieResponse(
            s_cookie_response::ServerboundCookieResponse {
              key: p.key,
              payload: None,
            },
          ))
          .await?;
      }
      _ => {}
    }
  }
}
