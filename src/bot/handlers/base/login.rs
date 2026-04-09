use std::io::{self, Error, ErrorKind};

use azalea_protocol::connect::Connection;
use azalea_protocol::packets::login::*;

/// Функция обработки всего цикла пакетов в состоянии Login
pub async fn process_login(conn: &mut Connection<ClientboundLoginPacket, ServerboundLoginPacket>) -> io::Result<()> {
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
        return Err(Error::new(ErrorKind::ConnectionAborted, format!("Disconnected (Login): {}", p.reason.to_string())));
      }
      ClientboundLoginPacket::CookieRequest(p) => {
        conn.write(ServerboundCookieResponse { key: p.key, payload: None }).await?;
      }
      _ => {}
    }
  }
}
