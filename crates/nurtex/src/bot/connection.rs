use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{RwLock, broadcast};

use crate::bot::capture::{capture_components, capture_connection};
use crate::bot::handlers::{ChatPayload, DisconnectPayload, Handlers};
use crate::bot::plugins::Plugins;
use crate::bot::types::PacketReader;
use crate::bot::{BotComponents, BotProfile};
use crate::protocol::connection::utils::handle_encryption_request;
use crate::protocol::connection::{ClientsidePacket, ConnectionState, NurtexConnection};
use crate::protocol::packets::play::{ClientsidePlayPacket, ServersideAcceptTeleportation, ServersideClientCommand};
use crate::protocol::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks},
  handshake::{ServersideGreet, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart},
  play::ServersidePlayPacket,
};
use crate::protocol::types::{ClientCommand, ClientIntention, ResourcePackState, Rotation, Vector3};
use crate::proxy::Proxy;
use crate::storage::Storage;
use crate::swarm::Speedometer;
use crate::world::Entity;

/// Функция спавна процесса подключения
pub async fn spawn_connection(
  connection: &Arc<RwLock<Option<NurtexConnection>>>,
  profile: &Arc<RwLock<BotProfile>>,
  components: &Arc<RwLock<BotComponents>>,
  speedometer: &Option<Arc<Speedometer>>,
  plugins: &Plugins,
  reader_tx: &PacketReader,
  storage: &Arc<Storage>,
  protocol_version: i32,
  coonnection_timeout: u64,
  proxy: &Arc<RwLock<Option<Proxy>>>,
  host: &str,
  port: u16,
  handlers: &Arc<Handlers>,
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
      Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed to receive a response from server within specified timeout")),
    },
    None => match tokio::time::timeout(Duration::from_millis(coonnection_timeout), NurtexConnection::new(host, port)).await {
      Ok(result) => match result {
        Ok(c) => c,
        Err(err) => return Err(err),
      },
      Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed to receive a response from server within specified timeout")),
    },
  };

  *connection.write().await = Some(conn);

  let profile_data = { profile.read().await.clone() };

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
        if let Some(handler) = &handlers.on_login_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(async move {
            let _ = handler_clone(username_clone).await;
          });
        }

        profile.write().await.uuid = p.uuid;

        capture_connection(&connection, async |conn| {
          conn.write_login_packet(ServersideLoginPacket::LoginAcknowledged(ServersideLoginAcknowledged)).await
        })
        .await?;

        break;
      }
      ClientsideLoginPacket::Disconnect(_p) => {
        if let Some(handler) = &handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(async move {
            let _ = handler_clone(username_clone, DisconnectPayload { state: ConnectionState::Login }).await;
          });
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
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
            .write_configuration_packet(ServersideConfigurationPacket::KeepAlive(crate::protocol::packets::configuration::MultisideKeepAlive {
              id: p.id,
            }))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::Ping(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::Pong(crate::protocol::packets::configuration::ServersidePong { id: p.id }))
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
              crate::protocol::packets::configuration::ServersideResourcePackResponse {
                uuid: p.uuid,
                state: ResourcePackState::Accepted,
              },
            ))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::Disconnect(_p) => {
        if let Some(handler) = &handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(async move {
            let _ = handler_clone(
              username_clone,
              DisconnectPayload {
                state: ConnectionState::Configuration,
              },
            )
            .await;
          });
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
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
    speedometer.bot_joined(profile_data.username.clone());
  }

  if let Some(handler) = &handlers.on_spawn_handler {
    let username_clone = profile_data.username.clone();
    let handler_clone = Arc::clone(handler);

    tokio::spawn(async move {
      let _ = handler_clone(username_clone).await;
    });
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
      Ok(Err(broadcast::error::RecvError::Closed)) => return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server")),
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
        let entity = Entity {
          entity_type: p.entity_type,
          entity_uuid: p.entity_uuid,
          position: p.position,
          rotation: Rotation::from_angle(p.yaw_angle, p.pitch_angle),
          velocity: p.velocity.to_vector3(),
          ..Default::default()
        };

        storage.add_entity(p.entity_id, entity).await;
      }
      ClientsidePlayPacket::RemoveEntities(p) => {
        storage
          .capture_entities(async |entities| {
            p.entities.iter().for_each(|entity_id| {
              entities.remove(entity_id);
            });
          })
          .await;
      }
      ClientsidePlayPacket::EntityPositionSync(p) => {
        storage
          .capture_entity(&p.entity_id, async |entity| {
            entity.position = p.position;
            entity.rotation = p.rotation;
            entity.velocity = p.velocity;
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityPos(p) => {
        storage
          .capture_entity(&p.entity_id, async |entity| {
            // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityRot(p) => {
        storage
          .capture_entity(&p.entity_id, async |entity| {
            entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityPosRot(p) => {
        storage
          .capture_entity(&p.entity_id, async |entity| {
            // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
            entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::SetEntityVelocity(p) => {
        storage
          .capture_entity(&p.entity_id, async |entity| {
            entity.position.with_velocity(p.velocity.to_vector3());
            entity.velocity = Vector3::from_lp_vector3(p.velocity);
          })
          .await;
      }
      ClientsidePlayPacket::KeepAlive(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::KeepAlive(crate::protocol::packets::play::MultisideKeepAlive { id: p.id }))
            .await
        })
        .await?;
      }
      ClientsidePlayPacket::PlayerChat(p) => {
        if let Some(handler) = &handlers.on_chat_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(async move {
            let _ = handler_clone(
              username_clone,
              ChatPayload {
                message: p.message,
                sender_uuid: p.sender_uuid,
              },
            )
            .await;
          });
        }
      }
      ClientsidePlayPacket::Ping(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::Pong(crate::protocol::packets::play::ServersidePong { id: p.id }))
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
            .write_play_packet(ServersidePlayPacket::ResourcePackResponse(crate::protocol::packets::play::ServersideResourcePackResponse {
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
        if let Some(handler) = &handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(async move {
            let _ = handler_clone(username_clone, DisconnectPayload { state: ConnectionState::Play }).await;
          });
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
      }
      _ => {}
    }
  }
}