#[cfg(test)]
mod tests {
  use std::io;

  use nurtex_protocol::connection::utils::handle_encryption_request;
  use nurtex_protocol::connection::{ConnectionState, NurtexConnection};
  use nurtex_protocol::packets::configuration::{
    ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideClientInformation, ServersideConfigurationPacket, ServersideKnownPacks,
    ServersideResourcePackResponse,
  };
  use nurtex_protocol::packets::handshake::{ServersideGreet, ServersideHandshakePacket};
  use nurtex_protocol::packets::login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart};
  use nurtex_protocol::packets::play::{ClientsidePlayPacket, ServersidePlayPacket};
  use nurtex_protocol::types::{AccurateHand, ChatMode, ClientIntention, DisplayedSkinParts, ParticleStatus, ResourcePackState};

  #[tokio::test]
  async fn test_client() -> io::Result<()> {
    let conn = match NurtexConnection::new("localhost", 25565).await {
      Ok(c) => c,
      Err(_) => return Ok(()),
    };

    conn
      .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
        protocol_version: 774,
        server_host: "localhost".to_string(),
        server_port: 25565,
        intention: ClientIntention::Login,
      }))
      .await?;

    conn.set_state(ConnectionState::Login).await;

    conn
      .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart {
        username: "TestBot".to_string(),
        uuid: uuid::Uuid::nil(),
      }))
      .await?;

    loop {
      if let Some(p) = conn.read_login_packet().await {
        match p {
          ClientsideLoginPacket::Compression(p) => {
            conn.set_compression_threshold(p.compression_threshold).await;
          }
          ClientsideLoginPacket::EncryptionRequest(request) => {
            if let Some((response, secret_key)) = handle_encryption_request(&request) {
              conn.write_login_packet(ServersideLoginPacket::EncryptionResponse(response)).await?;
              conn.set_encryption_key(secret_key).await;
            }
          }
          ClientsideLoginPacket::LoginSuccess(_p) => {
            conn.write_login_packet(ServersideLoginPacket::LoginAcknowledged(ServersideLoginAcknowledged)).await?;
            break;
          }
          _ => {}
        }
      } else {
        break;
      }
    }

    conn.set_state(ConnectionState::Configuration).await;

    conn
      .write_configuration_packet(ServersideConfigurationPacket::ClientInformation(ServersideClientInformation {
        locale: "en_US".to_string(),
        view_distance: 10,
        chat_mode: ChatMode::Enabled,
        chat_colors: true,
        displayed_skin_parts: DisplayedSkinParts::default(),
        main_hand: AccurateHand::Right,
        enable_text_filtering: false,
        allow_server_listings: true,
        particle_status: ParticleStatus::Minimal,
      }))
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

    conn.set_state(ConnectionState::Play).await;

    loop {
      if let Some(p) = conn.read_play_packet().await {
        println!("Получен пакет: {:?}", p);

        match p {
          ClientsidePlayPacket::KeepAlive(p) => {
            conn
              .write_play_packet(ServersidePlayPacket::KeepAlive(nurtex_protocol::packets::play::MultisideKeepAlive { id: p.id }))
              .await?;

            // conn
            //  .write_play_packet(ServersidePlayPacket::ClientCommand(ServersideClientCommand { command: ClientCommand::PerformRespawn }))
            //  .await?;
          }
          ClientsidePlayPacket::Ping(p) => {
            conn
              .write_play_packet(ServersidePlayPacket::Pong(nurtex_protocol::packets::play::ServersidePong { id: p.id }))
              .await?;
          }
          ClientsidePlayPacket::Disconnect(_p) => {
            println!("Клиент был отключен");
            break;
          }
          _ => {}
        }
      }
    }

    Ok(())
  }
}
