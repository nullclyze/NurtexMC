#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::HumanoidArm;
  use nurtex::common::client_information::ClientInformation;
  use nurtex::core::bot::{AutoReconnectPlugin, Bot, BotPlugins};
  use nurtex::core::events::EventHandler;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let bot = Bot::new("NurtexBot");

    let mut event_handler = EventHandler::new();

    event_handler.on_spawn(|terminal| async move {
      let username = terminal.receiver;
      println!("Bot {} spawned!", username);
    });

    event_handler.on_chat(|_terminal, _sender, msg| async move {
      println!("Chat: {}", msg);
    });

    bot
      .set_event_handler(event_handler)
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
      })
      .connect_to("localhost", 25565)
      .await?;

    Ok(())
  }
}
