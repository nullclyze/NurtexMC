use std::io::{self, Error, ErrorKind};
use std::pin::Pin;

use azalea_core::direction::Direction;
use azalea_core::position::{BlockPos, Vec3};
use azalea_entity::LookDirection;
use azalea_protocol::common::movements::MoveFlags;
use azalea_protocol::packets::game::s_player_action::Action;
use azalea_protocol::packets::game::{
  ServerboundGamePacket, ServerboundMovePlayerPos, ServerboundMovePlayerRot,
  ServerboundPlayerAction, ServerboundSwing, ServerboundUseItem,
};

use crate::core::bot::Bot;
use crate::core::common::BotCommand;
use crate::core::events::BotEvent;

/// Тип обработчика команд
pub type CommandProcessorFn =
  for<'a> fn(
    &'a mut Bot,
    BotCommand,
  ) -> Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + 'a>>;

/// Дефолтный обработчик команд
pub fn default_command_processor(
  bot: &mut Bot,
  command: BotCommand,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<bool>> + Send + '_>> {
  Box::pin(process_command(bot, command))
}

/// Функция обработки внешней команды
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
