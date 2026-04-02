use std::io::{self, Error, ErrorKind};
use std::pin::Pin;

use azalea_core::position::Vec3;
use azalea_entity::LookDirection;
use azalea_protocol::packets::game::{
  ClientboundGamePacket, ServerboundAcceptTeleportation, ServerboundClientCommand,
  ServerboundGamePacket, ServerboundKeepAlive, ServerboundPong,
};

use crate::core::bot::Bot;
use crate::core::data::{Entity, PlayerInfo};
use crate::core::events::{BotEvent, ChatPayload};
use crate::utils::timestamp;

/// Тип обработчика пакетов
pub type PacketProcessorFn =
  for<'a> fn(
    &'a mut Bot,
    ClientboundGamePacket,
  ) -> Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + 'a>>;

/// Дефолтный обработчик пакетов
pub fn default_packet_processor(
  bot: &mut Bot,
  packet: ClientboundGamePacket,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + '_>> {
  Box::pin(process_packet(bot, packet))
}

/// Функция обработки пакета (в состоянии Play)
async fn process_packet(bot: &mut Bot, packet: ClientboundGamePacket) -> io::Result<bool> {
  let Some(conn) = &mut bot.connection else {
    return Err(Error::new(
      ErrorKind::NotConnected,
      format!("Bot {} connection could not be obtained", bot.username),
    ));
  };

  match packet {
    ClientboundGamePacket::AddEntity(p) => {
      let entity = Entity {
        entity_type: p.entity_type.to_string(),
        uuid: p.uuid,
        position: p.position,
        velocity: Vec3::ZERO,
        look_direction: LookDirection::new(p.y_rot.into(), p.x_rot.into()),
        on_ground: false,
        player_info: None,
      };

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            guard.entities.insert(p.id.0, entity);
          }
          Err(_) => {}
        }
      } else {
        bot
          .local_storage
          .write()
          .await
          .entities
          .insert(p.id.0, entity);
      }
    }
    ClientboundGamePacket::RemoveEntities(p) => {
      let mut ids = Vec::new();
      p.entity_ids.iter().for_each(|id| ids.push(id.0));

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            guard.entities.retain(|id, _| !ids.contains(id));
          }
          Err(_) => {}
        }
      } else {
        bot
          .local_storage
          .write()
          .await
          .entities
          .retain(|id, _| !ids.contains(id));
      }
    }
    ClientboundGamePacket::Login(p) => {
      let profile = &mut bot.components.profile;

      profile.entity_id = Some(p.player_id.0);
      profile.game_mode = p.common.game_type.name().to_string();

      if p.show_death_screen && bot.plugins.auto_respawn.enabled {
        conn
          .write(ServerboundGamePacket::ClientCommand(
            ServerboundClientCommand {
              action: azalea_protocol::packets::game::s_client_command::Action::PerformRespawn,
            },
          ))
          .await?;
      }

      bot.emit_event(BotEvent::Spawn);
    }
    ClientboundGamePacket::MoveEntityPos(p) => {
      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.entity_id.0) {
              entity.position += p.delta.into();
              entity.on_ground = p.on_ground;
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot
          .local_storage
          .write()
          .await
          .entities
          .get_mut(&p.entity_id.0)
        {
          entity.position += p.delta.into();
          entity.on_ground = p.on_ground;
        }
      }
    }
    ClientboundGamePacket::MoveEntityRot(p) => {
      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.entity_id.0) {
              entity.on_ground = p.on_ground;

              let y_rot = entity.look_direction.y_rot() + p.y_rot as f32;
              let x_rot = entity.look_direction.x_rot() + p.x_rot as f32;

              entity.look_direction = LookDirection::new(y_rot, x_rot);
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot
          .local_storage
          .write()
          .await
          .entities
          .get_mut(&p.entity_id.0)
        {
          entity.on_ground = p.on_ground;

          let y_rot = entity.look_direction.y_rot() + p.y_rot as f32;
          let x_rot = entity.look_direction.x_rot() + p.x_rot as f32;

          entity.look_direction = LookDirection::new(y_rot, x_rot);
        }
      }
    }
    ClientboundGamePacket::MoveEntityPosRot(p) => {
      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.entity_id.0) {
              entity.position += p.delta.into();
              entity.on_ground = p.on_ground;

              let y_rot = entity.look_direction.y_rot() + p.y_rot as f32;
              let x_rot = entity.look_direction.x_rot() + p.x_rot as f32;

              entity.look_direction = LookDirection::new(y_rot, x_rot);
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot
          .local_storage
          .write()
          .await
          .entities
          .get_mut(&p.entity_id.0)
        {
          entity.position += p.delta.into();
          entity.on_ground = p.on_ground;

          let y_rot = entity.look_direction.y_rot() + p.y_rot as f32;
          let x_rot = entity.look_direction.x_rot() + p.x_rot as f32;

          entity.look_direction = LookDirection::new(y_rot, x_rot);
        }
      }
    }
    ClientboundGamePacket::PlayerInfoUpdate(p) => {
      let profile = &mut bot.components.profile;

      for entry in p.entries {
        if entry.profile.name == bot.username {
          profile.ping = entry.latency;
        } else {
          if let Some(shared_storage) = &bot.shared_storage {
            match shared_storage.try_write() {
              Ok(mut guard) => {
                for (_id, entity) in &mut guard.entities {
                  if entity.uuid != entry.profile.uuid {
                    continue;
                  }

                  let player_info = PlayerInfo {
                    username: entry.profile.name.clone(),
                    game_mode: entry.game_mode.name().to_string(),
                    ping: entry.latency,
                  };

                  entity.player_info = Some(player_info);
                }
              }
              Err(_) => {}
            }
          } else {
            let storage = &mut bot.local_storage;

            for (_id, entity) in &mut storage.write().await.entities {
              if entity.uuid != entry.profile.uuid {
                continue;
              }

              let player_info = PlayerInfo {
                username: entry.profile.name.clone(),
                game_mode: entry.game_mode.name().to_string(),
                ping: entry.latency,
              };

              entity.player_info = Some(player_info);
            }
          }
        }
      }
    }
    ClientboundGamePacket::SetHealth(p) => {
      let state = &mut bot.components.state;
      state.health = p.health;
      state.satiety = p.food;
      state.saturation = p.saturation;
    }
    ClientboundGamePacket::PlayerRotation(p) => {
      bot.components.physics.look_direction = LookDirection::new(p.y_rot, p.x_rot);
    }
    ClientboundGamePacket::PlayerPosition(p) => {
      let physics = &mut bot.components.physics;

      physics.position = p.change.pos;

      let delta_length_squared = p.change.delta.length_squared();

      if delta_length_squared > 0.0001 {
        physics.velocity = p.change.delta;
      }

      physics.look_direction = p.change.look_direction;

      conn
        .write(ServerboundGamePacket::AcceptTeleportation(
          ServerboundAcceptTeleportation { id: p.id },
        ))
        .await?;
    }
    ClientboundGamePacket::SetEntityMotion(p) => {
      if bot.is_this_my_entity_id(p.id.0) {
        let physics = &mut bot.components.physics;
        physics.velocity = p.delta.to_vec3();
      } else {
        if let Some(shared_storage) = &bot.shared_storage {
          match shared_storage.try_write() {
            Ok(mut guard) => {
              if let Some(entity) = guard.entities.get_mut(&p.id.0) {
                entity.velocity = p.delta.to_vec3();
              }
            }
            Err(_) => {}
          }
        } else {
          if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.id.0) {
            entity.velocity = p.delta.to_vec3();
          }
        }
      }
    }
    ClientboundGamePacket::EntityPositionSync(p) => {
      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.id.0) {
              entity.position = p.values.pos;
              entity.velocity = p.values.delta;
              entity.look_direction = p.values.look_direction;
              entity.on_ground = p.on_ground;
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.id.0) {
          entity.position = p.values.pos;
          entity.velocity = p.values.delta;
          entity.look_direction = p.values.look_direction;
          entity.on_ground = p.on_ground;
        }
      }
    }
    ClientboundGamePacket::KeepAlive(p) => {
      conn
        .write(ServerboundGamePacket::KeepAlive(ServerboundKeepAlive {
          id: p.id,
        }))
        .await?;
    }
    ClientboundGamePacket::Ping(p) => {
      conn
        .write(ServerboundGamePacket::Pong(ServerboundPong { id: p.id }))
        .await?;
    }
    ClientboundGamePacket::PlayerCombatKill(_p) => {
      if bot.plugins.auto_respawn.enabled {
        conn
          .write(ServerboundGamePacket::ClientCommand(
            ServerboundClientCommand {
              action: azalea_protocol::packets::game::s_client_command::Action::PerformRespawn,
            },
          ))
          .await?;
      }

      bot.emit_event(BotEvent::Death);
    }
    ClientboundGamePacket::SystemChat(p) => {
      bot.emit_event(BotEvent::Chat(ChatPayload {
        sender_uuid: None,
        message: p.content.to_string(),
        timestamp: timestamp(),
      }));
    }
    ClientboundGamePacket::PlayerChat(p) => {
      bot.emit_event(BotEvent::Chat(ChatPayload {
        sender_uuid: Some(p.sender),
        message: p.message().to_string(),
        timestamp: timestamp(),
      }));
    }
    ClientboundGamePacket::Disconnect(p) => {
      return Err(Error::new(
        ErrorKind::ConnectionAborted,
        format!("Bot was disconnected (play): {}", p.reason.to_string()),
      ));
    }
    _ => return Ok(true),
  }

  Ok(true)
}
