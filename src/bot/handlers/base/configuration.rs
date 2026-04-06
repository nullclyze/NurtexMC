use std::io::{self, Error, ErrorKind};

use azalea_buf::AzaleaWrite;
use azalea_protocol::connect::Connection;
use azalea_protocol::packets::config::*;

use crate::bot::Bot;

/// Функция обработки всего цикла пакетов в состоянии Configuration
pub async fn process_configuration(
  bot: &mut Bot,
  conn: &mut Connection<ClientboundConfigPacket, ServerboundConfigPacket>,
) -> io::Result<()> {
  let info = bot.get_information_ref();

  let mut brand_data = Vec::new();

  info.brand.azalea_write(&mut brand_data).unwrap();

  conn
    .write(ServerboundConfigPacket::CustomPayload(
      s_custom_payload::ServerboundCustomPayload {
        identifier: "brand".into(),
        data: brand_data.into(),
      },
    ))
    .await?;

  conn
    .write(ServerboundConfigPacket::ClientInformation(
      s_client_information::ServerboundClientInformation {
        information: info.client.clone(),
      },
    ))
    .await?;

  loop {
    let packet = match conn.read().await {
      Ok(p) => p,
      Err(_) => continue,
    };

    match packet {
      ClientboundConfigPacket::SelectKnownPacks(_) => {
        conn
          .write(ServerboundConfigPacket::SelectKnownPacks(
            s_select_known_packs::ServerboundSelectKnownPacks {
              known_packs: vec![],
            },
          ))
          .await?;
      }
      ClientboundConfigPacket::ResourcePackPush(p) => {
        conn
          .write(ServerboundConfigPacket::ResourcePack(
            s_resource_pack::ServerboundResourcePack {
              id: p.id,
              action: s_resource_pack::Action::Accepted,
            },
          ))
          .await?;
        conn
          .write(ServerboundConfigPacket::ResourcePack(
            s_resource_pack::ServerboundResourcePack {
              id: p.id,
              action: s_resource_pack::Action::SuccessfullyLoaded,
            },
          ))
          .await?;
      }
      ClientboundConfigPacket::Ping(p) => {
        conn
          .write(ServerboundConfigPacket::Pong(s_pong::ServerboundPong {
            id: p.id,
          }))
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
          format!("Disconnected (Configuration): {}", p.reason.to_string()),
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
