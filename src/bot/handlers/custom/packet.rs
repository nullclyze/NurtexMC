use std::io::{self, Error, ErrorKind};
use std::pin::Pin;
use std::sync::Arc;

use azalea_core::position::{BlockPos, ChunkPos};
use azalea_protocol::packets::game::{
  ClientboundGamePacket, ServerboundAcceptTeleportation, ServerboundClientCommand, ServerboundGamePacket, ServerboundKeepAlive, ServerboundPong,
  s_resource_pack::ServerboundResourcePack,
};

use crate::bot::Bot;
use crate::bot::components::position::Position;
use crate::bot::components::rotation::Rotation;
use crate::bot::components::velocity::Velocity;
use crate::bot::events::{BotEvent, ChatPayload};
use crate::bot::events::{ChunkPayload, DisconnectPayload};
use crate::bot::transmitter::BotPackage;
use crate::bot::world::{Entity, PlayerInfo};
use crate::utils::time::timestamp;

/// Тип обработчика пакетов
pub type PacketProcessorFn<P> = for<'a> fn(&'a mut Bot<P>, Arc<ClientboundGamePacket>) -> Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + 'a>>;

/// Дефолтный обработчик пакетов
pub fn default_packet_processor<P: BotPackage>(
  bot: &mut Bot<P>,
  packet: Arc<ClientboundGamePacket>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + '_>> {
  Box::pin(process_packet(bot, packet))
}

/// Функция обработки пакета (в состоянии Play)
async fn process_packet<P: BotPackage>(bot: &mut Bot<P>, packet: Arc<ClientboundGamePacket>) -> io::Result<bool> {
  match &*packet {
    ClientboundGamePacket::AddEntity(p) => {
      let entity = Entity {
        entity_type: p.entity_type.to_string(),
        uuid: p.uuid,
        position: Position::from_vec3(p.position),
        velocity: Velocity::zero(),
        rotation: Rotation::new(p.y_rot.into(), p.x_rot.into()),
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
        bot.local_storage.write().await.entities.insert(p.id.0, entity);
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
        bot.local_storage.write().await.entities.retain(|id, _| !ids.contains(id));
      }
    }
    ClientboundGamePacket::LevelChunkWithLight(p) => {
      let chunk_data = p.chunk_data.data.to_vec();

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            guard.load_chunk(p.x, p.z, chunk_data);

            bot.emit_event(BotEvent::ChunkLoaded(ChunkPayload {
              x: p.x,
              z: p.z,
              storage: shared_storage.clone(),
            }));
          }
          Err(_) => {}
        }
      } else {
        bot.local_storage.write().await.load_chunk(p.x, p.z, chunk_data);

        bot.emit_event(BotEvent::ChunkLoaded(ChunkPayload {
          x: p.x,
          z: p.z,
          storage: bot.local_storage.clone(),
        }));
      }
    }
    ClientboundGamePacket::ForgetLevelChunk(p) => {
      let chunk_pos = ChunkPos::new(p.pos.x, p.pos.z);

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            guard.remove_chunk(&chunk_pos);
          }
          Err(_) => {}
        }
      } else {
        bot.local_storage.write().await.remove_chunk(&chunk_pos);
      }
    }
    ClientboundGamePacket::BlockUpdate(p) => {
      let pos = BlockPos::new(p.pos.x, p.pos.y, p.pos.z);
      let block_state = p.block_state.id() as u32;

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            guard.set_block(&pos, block_state);
          }
          Err(_) => {}
        }
      } else {
        bot.local_storage.write().await.set_block(&pos, block_state);
      }
    }
    ClientboundGamePacket::SectionBlocksUpdate(p) => {
      for state in &p.states {
        let local_x = state.pos.x as i32;
        let local_y = state.pos.y as i32;
        let local_z = state.pos.z as i32;

        let pos = BlockPos::new((p.section_pos.x << 4) + local_x, (p.section_pos.y << 4) + local_y, (p.section_pos.z << 4) + local_z);

        let block_state = state.state.id() as u32;

        if let Some(shared_storage) = &bot.shared_storage {
          match shared_storage.try_write() {
            Ok(mut guard) => {
              guard.set_block(&pos, block_state);
            }
            Err(_) => {}
          }
        } else {
          bot.local_storage.write().await.set_block(&pos, block_state);
        }
      }
    }
    ClientboundGamePacket::Login(p) => {
      let profile = &mut bot.components.profile;

      profile.entity_id = Some(p.player_id.0);
      profile.game_mode = p.common.game_type.name().to_string();

      if p.show_death_screen && bot.plugins.auto_respawn.enabled {
        let Some(conn) = &mut bot.connection else {
          return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
        };

        conn
          .write(ServerboundGamePacket::ClientCommand(ServerboundClientCommand {
            action: azalea_protocol::packets::game::s_client_command::Action::PerformRespawn,
          }))
          .await?;
      }

      bot.emit_event(BotEvent::Spawn);
    }
    ClientboundGamePacket::MoveEntityPos(p) => {
      if bot.is_this_my_entity_id(p.entity_id.0) {
        return Ok(true);
      }

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.entity_id.0) {
              entity.position.apply_velocity(Velocity::from_vec3(p.delta.clone().into()));
              entity.on_ground = p.on_ground;
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.entity_id.0) {
          entity.position.apply_velocity(Velocity::from_vec3(p.delta.clone().into()));
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

              let yaw = entity.rotation.yaw + p.y_rot as f32;
              let pitch = entity.rotation.pitch + p.x_rot as f32;

              entity.rotation = Rotation::new(yaw, pitch);
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.entity_id.0) {
          entity.on_ground = p.on_ground;

          let yaw = entity.rotation.yaw + p.y_rot as f32;
          let pitch = entity.rotation.pitch + p.x_rot as f32;

          entity.rotation = Rotation::new(yaw, pitch);
        }
      }
    }
    ClientboundGamePacket::MoveEntityPosRot(p) => {
      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.entity_id.0) {
              entity.position.apply_velocity(Velocity::from_vec3(p.delta.clone().into()));
              entity.on_ground = p.on_ground;

              let yaw = entity.rotation.yaw + p.y_rot as f32;
              let pitch = entity.rotation.pitch + p.x_rot as f32;

              entity.rotation = Rotation::new(yaw, pitch);
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.entity_id.0) {
          entity.position.apply_velocity(Velocity::from_vec3(p.delta.clone().into()));
          entity.on_ground = p.on_ground;

          let yaw = entity.rotation.yaw + p.y_rot as f32;
          let pitch = entity.rotation.pitch + p.x_rot as f32;

          entity.rotation = Rotation::new(yaw, pitch);
        }
      }
    }
    ClientboundGamePacket::PlayerInfoUpdate(p) => {
      let profile = &mut bot.components.profile;

      for entry in &p.entries {
        if entry.profile.name == bot.account.username {
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
      bot.update_rotation(Rotation::new(p.y_rot, p.x_rot));
    }
    ClientboundGamePacket::PlayerPosition(p) => {
      bot.update_position(Position::new(p.change.pos.x, p.change.pos.y, p.change.pos.z));
      bot.update_rotation(Rotation::from_look_direction(p.change.look_direction));

      let Some(conn) = &mut bot.connection else {
        return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
      };

      conn.write(ServerboundGamePacket::AcceptTeleportation(ServerboundAcceptTeleportation { id: p.id })).await?;
    }
    ClientboundGamePacket::SetEntityMotion(p) => {
      if bot.is_this_my_entity_id(p.id.0) {
        bot.components.velocity = Velocity::from_vec3(p.delta.to_vec3());
      } else {
        if let Some(shared_storage) = &bot.shared_storage {
          match shared_storage.try_write() {
            Ok(mut guard) => {
              if let Some(entity) = guard.entities.get_mut(&p.id.0) {
                entity.velocity = Velocity::from_vec3(p.delta.to_vec3());
              }
            }
            Err(_) => {}
          }
        } else {
          if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.id.0) {
            entity.velocity = Velocity::from_vec3(p.delta.to_vec3());
          }
        }
      }
    }
    ClientboundGamePacket::EntityPositionSync(p) => {
      if bot.is_this_my_entity_id(p.id.0) {
        return Ok(true);
      }

      if let Some(shared_storage) = &bot.shared_storage {
        match shared_storage.try_write() {
          Ok(mut guard) => {
            if let Some(entity) = guard.entities.get_mut(&p.id.0) {
              entity.position = Position::from_vec3(p.values.pos);
              entity.velocity = Velocity::from_vec3(p.values.delta);
              entity.rotation = Rotation::from_look_direction(p.values.look_direction);
              entity.on_ground = p.on_ground;
            }
          }
          Err(_) => {}
        }
      } else {
        if let Some(entity) = bot.local_storage.write().await.entities.get_mut(&p.id.0) {
          entity.position = Position::from_vec3(p.values.pos);
          entity.velocity = Velocity::from_vec3(p.values.delta);
          entity.rotation = Rotation::from_look_direction(p.values.look_direction);
          entity.on_ground = p.on_ground;
        }
      }
    }
    ClientboundGamePacket::KeepAlive(p) => {
      let Some(conn) = &mut bot.connection else {
        return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
      };

      conn.write(ServerboundGamePacket::KeepAlive(ServerboundKeepAlive { id: p.id })).await?;
    }
    ClientboundGamePacket::Ping(p) => {
      let Some(conn) = &mut bot.connection else {
        return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
      };

      conn.write(ServerboundGamePacket::Pong(ServerboundPong { id: p.id })).await?;
    }
    ClientboundGamePacket::PlayerCombatKill(_p) => {
      bot.emit_event(BotEvent::Death);

      if bot.plugins.auto_respawn.enabled {
        let Some(conn) = &mut bot.connection else {
          return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
        };

        conn
          .write(ServerboundGamePacket::ClientCommand(ServerboundClientCommand {
            action: azalea_protocol::packets::game::s_client_command::Action::PerformRespawn,
          }))
          .await?;
      }
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
      bot.emit_event(BotEvent::Disconnect(DisconnectPayload {
        reason: p.reason.to_string(),
        timestamp: timestamp(),
      }));

      return Err(Error::new(ErrorKind::ConnectionAborted, format!("Disconnected (Play): {}", p.reason.to_string())));
    }
    ClientboundGamePacket::ResourcePackPush(p) => {
      let Some(conn) = &mut bot.connection else {
        return Err(Error::new(ErrorKind::NotConnected, "Connection could not be obtained"));
      };
      conn
        .write(ServerboundGamePacket::ResourcePack(ServerboundResourcePack {
          id: p.id,
          action: azalea_protocol::packets::game::s_resource_pack::Action::Accepted,
        }))
        .await?;
      conn
        .write(ServerboundGamePacket::ResourcePack(ServerboundResourcePack {
          id: p.id,
          action: azalea_protocol::packets::game::s_resource_pack::Action::SuccessfullyLoaded,
        }))
        .await?;
    }
    _ => return Ok(true),
  }

  Ok(true)
}
