#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::core::bot::{AutoReconnectPlugin, AutoRespawnPlugin, Bot, BotCommand, BotPlugins};
  use nurtex::core::events::BotEvent;
  use nurtex::packets::game::ClientboundGamePacket;
  use nurtex::utils::sleep;
  use uuid::Uuid;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let (mut bot, terminal) = Bot::new("NurtexBot", Uuid::nil());

    bot = bot
      .set_event_listener(event_listener)
      .set_plugins(BotPlugins {
        auto_reconnect: AutoReconnectPlugin {
          enabled: false,
          reconnect_delay: 0
        },
        auto_respawn: AutoRespawnPlugin {
          enabled: false,
        },
        ..Default::default()
      });

    tokio::spawn(async move {
      sleep(3000).await;
      terminal.send(BotCommand::Chat("Test".to_string())).await;
      sleep(5000).await; 
      terminal.send(BotCommand::Disconnect).await;
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
      },
      BotEvent::Chat { sender_uuid, message } => {
        if let Some(uuid) = sender_uuid {
          println!("Bot {} received a message from {}: {}", bot.username, uuid, message);
        }
      }
      BotEvent::Packet(packet) => match packet {
        ClientboundGamePacket::AddEntity(p) => {
          println!("Bot {} has received a new entity! Entity ID: {}, Position: {}", bot.username, p.id.0, p.position);
        }
        _ => {}
      }
      _ => {}
    }

    Ok(())
  }
}