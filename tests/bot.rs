#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::core::bot::Bot;
  use nurtex::core::events::EventHandler;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let bot = Bot::new("NurtexBot");

    let mut event_handler = EventHandler::new();

    event_handler.on_spawn(|terminal| async move {
      let username = &terminal.receiver;
      println!("Bot {} spawned!", username);
    });

    event_handler.on_chat(|_terminal, payload| async move {
      println!("[{}] Chat message: {}", payload.timestamp, payload.message);
    });

    bot
      .set_event_handler(event_handler)
      .connect_to("localhost", 25565)
      .await?;

    Ok(())
  }
}
