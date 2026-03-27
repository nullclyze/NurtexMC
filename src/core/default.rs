use std::io::{self, Error, ErrorKind};

use azalea_core::direction::Direction;
use azalea_core::position::{BlockPos, Vec3};
use azalea_entity::LookDirection;
use azalea_protocol::common::movements::MoveFlags;
use azalea_protocol::packets::game::s_player_action::Action;
use azalea_protocol::packets::game::{
  ClientboundGamePacket, ServerboundAcceptTeleportation, ServerboundClientCommand,
  ServerboundGamePacket, ServerboundKeepAlive, ServerboundMovePlayerPos, ServerboundMovePlayerRot,
  ServerboundPlayerAction, ServerboundPong, ServerboundSwing, ServerboundUseItem,
};

use crate::core::bot::{Bot, BotCommand};
use crate::core::data::{Entity, PlayerInfo};
use crate::core::events::BotEvent;

/// Дефолтный обработчик пакетов.
pub fn default_packet_processor(
  bot: &mut Bot,
  packet: ClientboundGamePacket,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + '_>> {
  Box::pin(process_packet(bot, packet))
}

/// Дефолтный обработчик команд.
pub fn default_command_processor(
  bot: &mut Bot,
  command: BotCommand,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + '_>> {
  Box::pin(process_command(bot, command))
}

/// Функция обработки пакета (в состоянии Play).
async fn process_packet(bot: &mut Bot, packet: ClientboundGamePacket) -> io::Result<bool> {
  let Some(conn) = &mut bot.connection else {
    return Err(Error::new(
      ErrorKind::NotConnected,
      format!("Bot {} connection could not be obtained", bot.username),
    ));
  };

  match packet {
    ClientboundGamePacket::AddEntity(p) => {
      let storage = &mut bot.storage;

      let entity = Entity {
        entity_type: p.entity_type.to_string(),
        uuid: p.uuid,
        position: p.position,
        velocity: Vec3::ZERO,
        look_direction: LookDirection::new(p.y_rot.into(), p.x_rot.into()),
        on_ground: false,
        player_info: None,
      };

      storage.entities.insert(p.id.0, entity);
    }
    ClientboundGamePacket::RemoveEntities(p) => {
      let storage = &mut bot.storage;

      for entity_id in p.entity_ids {
        storage.entities.remove(&entity_id.0);
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
      if let Some(entity) = bot.storage.entities.get_mut(&p.entity_id.0) {
        entity.position += p.delta.into();
        entity.on_ground = p.on_ground;
      }
    }
    ClientboundGamePacket::MoveEntityRot(p) => {
      if let Some(entity) = bot.storage.entities.get_mut(&p.entity_id.0) {
        entity.on_ground = p.on_ground;

        let old_y_rot = entity.look_direction.y_rot();
        let old_x_rot = entity.look_direction.x_rot();

        entity.look_direction =
          LookDirection::new(old_y_rot + p.y_rot as f32, old_x_rot + p.x_rot as f32);
      }
    }
    ClientboundGamePacket::MoveEntityPosRot(p) => {
      if let Some(entity) = bot.storage.entities.get_mut(&p.entity_id.0) {
        entity.position += p.delta.into();
        entity.on_ground = p.on_ground;

        let old_y_rot = entity.look_direction.y_rot();
        let old_x_rot = entity.look_direction.x_rot();

        entity.look_direction =
          LookDirection::new(old_y_rot + p.y_rot as f32, old_x_rot + p.x_rot as f32);
      }
    }
    ClientboundGamePacket::PlayerInfoUpdate(p) => {
      let profile = &mut bot.components.profile;

      for entry in p.entries {
        if entry.profile.name == bot.username {
          profile.ping = entry.latency;
        } else {
          let storage = &mut bot.storage;

          for (_id, entity) in &mut storage.entities {
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
        if let Some(entity) = bot.storage.entities.get_mut(&p.id.0) {
          entity.velocity = p.delta.to_vec3();
        }
      }
    }
    ClientboundGamePacket::EntityPositionSync(p) => {
      if let Some(entity) = bot.storage.entities.get_mut(&p.id.0) {
        entity.position = p.values.pos;
        entity.velocity = p.values.delta;
        entity.look_direction = p.values.look_direction;
        entity.on_ground = p.on_ground;
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
      bot.emit_event(BotEvent::Chat {
        sender_uuid: None,
        message: p.content.to_string(),
      });
    }
    ClientboundGamePacket::PlayerChat(p) => {
      bot.emit_event(BotEvent::Chat {
        sender_uuid: Some(p.sender),
        message: p.message().to_string(),
      });
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

/// Функция обработки внешней команды.
async fn process_command(bot: &mut Bot, command: BotCommand) -> io::Result<bool> {
  let Some(conn) = &mut bot.connection else {
    return Err(Error::new(
      ErrorKind::NotConnected,
      format!("Bot {} connection could not be obtained", bot.username),
    ));
  };

  match command {
    BotCommand::Chat(message) => {
      bot.chat(message).await?;
    }
    BotCommand::SetDirection { yaw, pitch } => {
      conn
        .write(ServerboundGamePacket::MovePlayerRot(
          ServerboundMovePlayerRot {
            look_direction: LookDirection::new(yaw, pitch),
            flags: MoveFlags {
              on_ground: bot.components.physics.on_ground,
              horizontal_collision: false,
            },
          },
        ))
        .await?;
    }
    BotCommand::SetPosition { x, y, z } => {
      conn
        .write(ServerboundGamePacket::MovePlayerPos(
          ServerboundMovePlayerPos {
            pos: Vec3::new(x, y, z),
            flags: MoveFlags {
              on_ground: bot.components.physics.on_ground,
              horizontal_collision: false,
            },
          },
        ))
        .await?;
    }
    BotCommand::SwingArm(hand) => {
      conn
        .write(ServerboundGamePacket::Swing(ServerboundSwing { hand }))
        .await?;
    }
    BotCommand::StartUseItem(hand) => {
      let look_direction = bot.components.physics.look_direction;

      conn
        .write(ServerboundGamePacket::UseItem(ServerboundUseItem {
          hand: hand,
          seq: 0,
          y_rot: look_direction.y_rot(),
          x_rot: look_direction.x_rot(),
        }))
        .await?;
    }
    BotCommand::ReleaseUseItem => {
      conn
        .write(ServerboundGamePacket::PlayerAction(
          ServerboundPlayerAction {
            action: Action::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            seq: 0,
          },
        ))
        .await?;
    }
    BotCommand::SendPacket(packet) => {
      conn.write(packet).await?;
    }
    BotCommand::Disconnect => {
      bot.disconnect().await?;
      bot.emit_event(BotEvent::Disconnect);
      return Ok(false);
    }
    BotCommand::Reconnect {
      server_host,
      server_port,
      interval,
    } => {
      bot.reconnect(&server_host, server_port, interval).await?;
    }
  }

  Ok(true)
}
