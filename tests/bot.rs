#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::bot::account::BotAccount;
  use nurtex::bot::Bot;
  use nurtex::bot::events::EventInvoker;

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let account = BotAccount::new("NurtexBot");

    let bot = Bot::create(account);

    let mut event_invoker = EventInvoker::new();

    event_invoker.on_spawn(|terminal| async move {
      println!("Бот {} заспавнился!", terminal.account.username);
    });

    event_invoker.on_chat(|terminal, payload| async move {
      println!(
        "Бот {} получил сообщение: {}",
        terminal.account.username, payload.message
      );
    });

    event_invoker.on_disconnect(|terminal, payload| async move {
      println!(
        "Бот {} отключился по причине: {}",
        terminal.account.username, payload.reason
      );
    });

    bot
      .set_event_invoker(event_invoker)
      .connect_to("localhost", 25565)
      .await?;

    Ok(())
  }
}
