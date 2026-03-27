#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::HumanoidArm;
  use nurtex::common::client_information::ClientInformation;
  use nurtex::core::bot::{AutoReconnectPlugin, Bot, BotPlugins};
  use nurtex::core::events::BotEvent;
  use nurtex::packets::game::ClientboundGamePacket;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let (mut bot, _terminal) = Bot::new("NurtexBot");

    bot = bot
      .set_event_listener(event_listener)
      .set_information(ClientInformation {
        main_hand: HumanoidArm::Left,
        ..Default::default()
      })
      .set_plugins(BotPlugins {
        auto_reconnect: AutoReconnectPlugin {
          enabled: false,
          reconnect_delay: 0,
        },
        ..Default::default()
      });

    bot.connect_to("localhost", 25565).await?;

    Ok(())
  }

  fn event_listener(bot: &mut Bot, event: BotEvent) -> io::Result<()> {
    match event {
      BotEvent::Spawn => {
        println!("Bot {} spawned!", bot.username);
      }
      BotEvent::Disconnect => {
        println!("Bot {} disconnected.", bot.username);
      }
      BotEvent::Chat {
        sender_uuid,
        message,
      } => {
        if let Some(uuid) = sender_uuid {
          println!(
            "Bot {} received a message from {}: {}",
            bot.username, uuid, message
          );
        }

        if message.contains("entities") {
          for (id, entity) in &bot.storage.entities {
            println!("{} - {:?}", id, entity);
          }
          
          println!("Entity count: {}", bot.storage.entities.len());
        }
      }
      BotEvent::Packet(packet) => match packet {
        ClientboundGamePacket::AddEntity(p) => {
          println!(
            "Bot {} has received a new entity! Entity ID: {}, Position: {}",
            bot.username, p.id.0, p.position
          );
        }
        _ => {}
      },
      _ => {}
    }

    Ok(())
  }
}
