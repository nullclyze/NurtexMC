#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::core::bot::Bot;
  use nurtex::core::events::EventInvoker;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let bot = Bot::new("NurtexBot");

    let mut event_invoker = EventInvoker::new();

    event_invoker.on_spawn(|terminal| async move {
      println!("Бот {} заспавнился!", terminal.receiver);
    });

    bot
      .set_event_invoker(event_invoker)
      .connect_to("localhost", 25565)
      .await?;

    Ok(())
  }
}
